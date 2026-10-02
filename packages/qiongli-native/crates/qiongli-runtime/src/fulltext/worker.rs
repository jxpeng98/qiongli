//! Same-executable parser child; no URL, filesystem path or credentials enter it.
use super::{FulltextError, FulltextOutput, MAX_BYTES, bounded_bytes, failure, parse_document};
use qiongli_bounded_alloc::BoundedAllocator;
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::io::{Read, Write};
use std::process::{Child, Command, ExitCode, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

const ARG: &str = "--qiongli-fulltext-worker-v1";
const HEAP_BYTES: usize = 512 * 1024 * 1024;
const OUTPUT_BYTES: usize = 16 * 1024 * 1024;
const TIMEOUT: Duration = Duration::from_secs(30);

#[global_allocator]
static ALLOCATOR: BoundedAllocator = BoundedAllocator::new();

/// Call before configuration/Host discovery at each executable entrypoint.
/// The parent remains unlimited; only a fresh parser child activates the ceiling.
pub fn run_worker_if_requested(args: &[OsString]) -> Option<ExitCode> {
    if args != [OsStr::new(ARG)] {
        return None;
    }
    if !ALLOCATOR.limit_worker_heap(HEAP_BYTES) {
        return Some(ExitCode::FAILURE);
    }
    if start_deadline(TIMEOUT).is_err() {
        return Some(ExitCode::FAILURE);
    }
    let result =
        bounded_bytes(std::io::stdin().lock()).and_then(|bytes| parse_document("", "", &bytes));
    Some(
        if serde_json::to_writer(std::io::stdout().lock(), &result).is_ok() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        },
    )
}

fn start_deadline(timeout: Duration) -> std::io::Result<()> {
    // Parent death does not run Reap::drop. The child must still exit even if
    // parsing blocks after stdin closes; a detached watchdog ends with it.
    std::thread::Builder::new().spawn(move || {
        std::thread::sleep(timeout);
        std::process::exit(1);
    })?;
    Ok(())
}

#[allow(
    clippy::disallowed_methods,
    reason = "launch only this native executable's fixed parser entrypoint, never an external runtime"
)]
pub(super) fn parse(
    source: &str,
    resolved: &str,
    bytes: Vec<u8>,
) -> Result<FulltextOutput, FulltextError> {
    if bytes.len() > MAX_BYTES {
        return Err(failure(
            "fulltext-too-large",
            "Fulltext exceeds the decoded byte limit",
        ));
    }
    let digest = format!("{:x}", Sha256::digest(&bytes));
    let binary = std::env::current_exe().map_err(|_| unavailable())?;
    let mut command = Command::new(binary);
    command.arg(ARG).env_clear().env("RAYON_NUM_THREADS", "1");
    let output = supervise(command, bytes, TIMEOUT, OUTPUT_BYTES)?;
    let mut doc = serde_json::from_slice::<Result<FulltextOutput, FulltextError>>(&output)
        .map_err(|_| {
            failure(
                "fulltext-worker-protocol",
                "Parser worker returned an invalid response",
            )
        })??;
    if doc.source_sha256 != digest || doc.total_segments != doc.segments.len() {
        return Err(failure(
            "fulltext-worker-protocol",
            "Parser worker source binding is invalid",
        ));
    }
    doc.source_url = source.into();
    doc.resolved_url = resolved.into();
    Ok(doc)
}

fn unavailable() -> FulltextError {
    failure(
        "fulltext-worker-unavailable",
        "Cannot start the bounded parser worker; use an available Host reader",
    )
}

struct Reap(Child);

impl Drop for Reap {
    fn drop(&mut self) {
        // The fixed worker never spawns subprocesses. Close its pipes on every
        // exit path, including timeout, output overflow and a failed supervisor.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn supervise(
    mut command: Command,
    bytes: Vec<u8>,
    timeout: Duration,
    max_output: usize,
) -> Result<Vec<u8>, FulltextError> {
    let started = Instant::now();
    let mut child = Reap(
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| unavailable())?,
    );
    let mut input = child.0.stdin.take().ok_or_else(unavailable)?;
    let output = child.0.stdout.take().ok_or_else(unavailable)?;
    std::thread::scope(|scope| {
        let writer = std::thread::Builder::new()
            .spawn_scoped(scope, move || input.write_all(&bytes))
            .map_err(|_| unavailable())?;
        let (send, receive) = mpsc::channel();
        let reader = std::thread::Builder::new().spawn_scoped(scope, move || {
            let mut bytes = Vec::new();
            let result = output
                .take(max_output as u64 + 1)
                .read_to_end(&mut bytes)
                .map(|_| bytes);
            let _ = send.send(result);
        });
        if reader.is_err() {
            let _ = child.0.kill();
            let _ = child.0.wait();
            return Err(unavailable());
        }
        let mut response = None;
        let result = loop {
            if response.is_none() {
                match receive.try_recv() {
                    Ok(Ok(bytes)) if bytes.len() <= max_output => response = Some(bytes),
                    Ok(_) | Err(mpsc::TryRecvError::Disconnected) => {
                        break Err(failure(
                            "fulltext-worker-output",
                            "Parser worker output failed or exceeded its limit",
                        ));
                    }
                    Err(mpsc::TryRecvError::Empty) => {}
                }
            }
            match child.0.try_wait() {
                Ok(Some(status)) if !status.success() => {
                    break Err(failure(
                        "fulltext-worker-failed",
                        "Parser worker exited before completion (resource limit or parser failure)",
                    ));
                }
                Ok(Some(_)) if response.is_some() => {
                    break Ok(response.take().expect("response checked"));
                }
                Err(_) => {
                    break Err(failure(
                        "fulltext-worker-failed",
                        "Cannot inspect parser worker completion",
                    ));
                }
                _ => {}
            }
            if started.elapsed() >= timeout {
                break Err(failure(
                    "fulltext-parse-timeout",
                    "Parser worker exceeded its time limit; use another source or Host reader",
                ));
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        // Terminate before joining pipe threads; otherwise a blocked writer or
        // reader could keep a timed-out request alive indefinitely.
        let _ = child.0.kill();
        let reaped = child.0.wait();
        let sent = writer.join();
        if reaped.is_err() {
            return Err(failure(
                "fulltext-worker-failed",
                "Cannot confirm parser worker cleanup",
            ));
        }
        match result {
            Ok(output) if matches!(sent, Ok(Ok(()))) => Ok(output),
            Ok(_) => Err(failure(
                "fulltext-worker-input",
                "Parser worker did not consume its complete input",
            )),
            error => error,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Invoked only by this test binary, never exposed by a production command.
    #[test]
    #[ignore]
    fn worker_probe() {
        match std::env::var("QIONGLI_WORKER_TEST").as_deref() {
            Ok("deadline") => {
                start_deadline(Duration::from_millis(100)).unwrap();
                std::thread::sleep(Duration::from_secs(5));
            }
            Ok("timeout") => std::thread::sleep(Duration::from_secs(5)),
            Ok("output") => std::io::stdout().write_all(&[b'x'; 16_384]).unwrap(),
            Ok("heap") => {
                assert!(ALLOCATOR.limit_worker_heap(16 * 1024 * 1024));
                let mut bytes = Vec::<u8>::new();
                assert!(bytes.try_reserve_exact(32 * 1024 * 1024).is_err());
                println!("heap limit refused allocation");
            }
            Ok("oom") => {
                assert!(ALLOCATOR.limit_worker_heap(16 * 1024 * 1024));
                std::hint::black_box(vec![0u8; 32 * 1024 * 1024]);
            }
            Ok("abort") => std::process::abort(),
            _ => println!("worker completed"),
        }
    }

    #[allow(clippy::disallowed_methods)]
    fn probe(action: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "fulltext::worker::tests::worker_probe",
                "--ignored",
                "--nocapture",
            ])
            .env("QIONGLI_WORKER_TEST", action);
        command
    }

    #[test]
    fn supervisor_handles_completion_heap_refusal_timeout_overflow_and_crash() {
        let start = Instant::now();
        assert!(!probe("deadline").status().unwrap().success());
        assert!(start.elapsed() < Duration::from_secs(5));
        for action in ["normal", "heap"] {
            let result =
                supervise(probe(action), Vec::new(), Duration::from_secs(10), 4096).unwrap();
            assert!(
                String::from_utf8(result)
                    .unwrap()
                    .contains(if action == "heap" {
                        "heap limit refused allocation"
                    } else {
                        "worker completed"
                    })
            );
        }
        for (action, timeout, code) in [
            (
                "timeout",
                Duration::from_millis(100),
                "fulltext-parse-timeout",
            ),
            ("output", Duration::from_secs(10), "fulltext-worker-output"),
            ("abort", Duration::from_secs(10), "fulltext-worker-failed"),
            ("oom", Duration::from_secs(10), "fulltext-worker-failed"),
        ] {
            let start = Instant::now();
            assert_eq!(
                supervise(probe(action), vec![0; 128 * 1024], timeout, 4096)
                    .unwrap_err()
                    .code,
                code
            );
            assert!(start.elapsed() < Duration::from_secs(10));
            assert!(supervise(probe("normal"), Vec::new(), Duration::from_secs(10), 4096).is_ok());
        }
    }
}
