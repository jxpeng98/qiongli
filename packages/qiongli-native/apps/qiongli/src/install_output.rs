//! Installation presentation only; approvals and commands retain their owners.
use std::io::{self, IsTerminal, Write};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DisplayOptions {
    pub(crate) verbose: bool,
    pub(crate) plain: bool,
}

pub(crate) trait InstallWriter: Write {
    fn verbose(&self) -> bool {
        false
    }

    fn progress(&mut self, _label: &str) -> Result<(), &'static str> {
        Ok(())
    }

    fn finish_progress(&mut self) -> Result<(), &'static str> {
        Ok(())
    }

    fn waiting(&mut self, elapsed: Duration, timeout: Duration) -> Result<(), &'static str> {
        if self.verbose() || elapsed.as_secs().is_multiple_of(30) {
            writeln!(
                self,
                "  Still running: {}s elapsed (limit {}s).",
                elapsed.as_secs(),
                timeout.as_secs()
            )
            .and_then(|()| self.flush())
            .map_err(|_| "installation-output-failed")?;
        }
        Ok(())
    }
}

#[cfg(test)]
impl InstallWriter for Vec<u8> {}

pub(crate) struct InstallOutput<W: Write> {
    inner: W,
    options: DisplayOptions,
    width: Option<fn() -> Option<usize>>,
    active: bool,
    last_wait: u64,
    phase: String,
}

impl<W: Write> InstallOutput<W> {
    pub(crate) fn terminal(inner: W, options: DisplayOptions) -> Self {
        let term = std::env::var("TERM").ok();
        let dynamic = supports_refresh(
            options,
            io::stdout().is_terminal(),
            term.as_deref(),
            std::env::var_os("CI").is_some() || std::env::var_os("NO_COLOR").is_some(),
        );
        Self {
            inner,
            options,
            width: dynamic.then_some(terminal_width),
            active: false,
            last_wait: 0,
            phase: "Working".into(),
        }
    }

    fn clear_active(&mut self) -> io::Result<()> {
        if self.active {
            // Only our current line: never erase scrollback, move up, hide the
            // cursor or enter raw/alternate-screen mode (including on Ctrl-C).
            self.inner.write_all(b"\r\x1b[2K")?;
            self.active = false;
        }
        Ok(())
    }

    fn draw(&mut self, label: &str) -> io::Result<bool> {
        self.clear_active()?;
        let Some(width) = self
            .width
            .and_then(|width| width())
            .filter(|width| *width >= 32)
        else {
            return Ok(false);
        };
        // Live text is ASCII and short, independent of command/path length.
        // Leave the final column unused to avoid terminal auto-wrap.
        let text: String = label
            .chars()
            .map(|ch| {
                if ch.is_ascii() && !ch.is_control() {
                    ch
                } else {
                    '?'
                }
            })
            .take((width - 1).min(28))
            .collect();
        self.active = true;
        self.inner.write_all(text.as_bytes())?;
        self.inner.flush()?;
        Ok(true)
    }
}

fn supports_refresh(
    options: DisplayOptions,
    terminal: bool,
    term: Option<&str>,
    plain_env: bool,
) -> bool {
    terminal
        && !options.plain
        && !options.verbose
        && !plain_env
        && term.is_some_and(|term| {
            ["xterm", "screen", "tmux", "rxvt", "ansi", "linux", "vt100"]
                .iter()
                .any(|known| term.starts_with(known))
        })
}

fn terminal_width() -> Option<usize> {
    #[cfg(unix)]
    {
        rustix::termios::tcgetwinsize(io::stdout())
            .ok()
            .map(|size| usize::from(size.ws_col))
    }
    #[cfg(not(unix))]
    {
        // Keep unsupported consoles append-only rather than assume ANSI mode.
        None
    }
}

impl<W: Write> Write for InstallOutput<W> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.clear_active()?;
        self.inner.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<W: Write> InstallWriter for InstallOutput<W> {
    fn verbose(&self) -> bool {
        self.options.verbose
    }

    fn progress(&mut self, label: &str) -> Result<(), &'static str> {
        self.last_wait = 0;
        self.phase = label
            .chars()
            .filter(|ch| ch.is_ascii() && !ch.is_control())
            .take(12)
            .collect();
        self.draw(label)
            .map(|_| ())
            .map_err(|_| "installation-output-failed")
    }

    fn finish_progress(&mut self) -> Result<(), &'static str> {
        self.clear_active()
            .and_then(|()| self.inner.flush())
            .map_err(|_| "installation-output-failed")
    }

    fn waiting(&mut self, elapsed: Duration, timeout: Duration) -> Result<(), &'static str> {
        let seconds = elapsed.as_secs();
        if self
            .draw(&format!("{} {seconds}s/{}s", self.phase, timeout.as_secs()))
            .map_err(|_| "installation-output-failed")?
        {
            return Ok(());
        }
        if self.options.verbose || seconds.saturating_sub(self.last_wait) >= 30 {
            self.last_wait = seconds;
            writeln!(
                self,
                "  Still running: {seconds}s elapsed (limit {}s).",
                timeout.as_secs()
            )
            .and_then(|()| self.flush())
            .map_err(|_| "installation-output-failed")?;
        }
        Ok(())
    }
}

impl<W: Write> Drop for InstallOutput<W> {
    fn drop(&mut self) {
        let _ = self.finish_progress();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(width: Option<fn() -> Option<usize>>) -> InstallOutput<Vec<u8>> {
        InstallOutput {
            inner: Vec::new(),
            options: DisplayOptions::default(),
            width,
            active: false,
            last_wait: 0,
            phase: "Working".into(),
        }
    }

    #[test]
    fn live_wait_does_not_add_rows_and_clears_before_prompts_and_results() {
        let mut output = output(Some(|| Some(40)));
        writeln!(output, "Approval preview stays visible").unwrap();
        output.progress("DSH 8/12").unwrap();
        for elapsed in [5, 10, 15] {
            output
                .waiting(Duration::from_secs(elapsed), Duration::from_secs(120))
                .unwrap();
        }
        write!(output, "Approve? ").unwrap();
        let text = String::from_utf8(output.inner.clone()).unwrap();
        assert_eq!(text.matches('\n').count(), 1);
        assert!(text.contains("DSH 8/12 15s/120s"));
        assert!(text.ends_with("\r\x1b[2KApprove? "));
        assert!(!text.contains("\x1b[1A") && !text.contains("\x1b[2J"));
        output.progress("next").unwrap();
        output.finish_progress().unwrap();
        assert!(!output.active);
    }

    #[test]
    fn plain_and_narrow_output_are_append_only_and_throttled() {
        for width in [None, Some((|| Some(20)) as fn() -> Option<usize>)] {
            let mut output = output(width);
            output.progress("DSH 8/12").unwrap();
            for elapsed in [5, 10, 30, 35, 61] {
                output
                    .waiting(Duration::from_secs(elapsed), Duration::from_secs(120))
                    .unwrap();
            }
            let text = String::from_utf8(output.inner.clone()).unwrap();
            assert!(!text.contains('\x1b') && !text.contains('\r'));
            assert_eq!(text.lines().count(), 2);
            assert!(text.contains("30s") && text.contains("61s"));
        }
        for (terminal, term, plain_env) in [
            (false, Some("xterm"), false),
            (true, None, false),
            (true, Some("dumb"), false),
            (true, Some("xterm"), true),
        ] {
            assert!(!supports_refresh(
                DisplayOptions::default(),
                terminal,
                term,
                plain_env
            ));
        }
        for options in [
            DisplayOptions {
                plain: true,
                verbose: false,
            },
            DisplayOptions {
                plain: false,
                verbose: true,
            },
        ] {
            assert!(!supports_refresh(options, true, Some("xterm"), false));
        }
    }

    #[test]
    fn live_status_bounds_and_escapes_text_and_propagates_output_failure() {
        let mut output = output(Some(|| Some(32)));
        output.progress(&"中文\x1b[2J\nlong".repeat(20)).unwrap();
        assert!(output.inner.len() < 32);
        assert!(!output.inner.contains(&b'\x1b') && !output.inner.contains(&b'\n'));
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::ErrorKind::BrokenPipe.into())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut broken = InstallOutput {
            inner: Broken,
            options: DisplayOptions::default(),
            width: Some(|| Some(80)),
            active: false,
            last_wait: 0,
            phase: "Working".into(),
        };
        assert_eq!(
            broken.progress("Working"),
            Err("installation-output-failed")
        );
    }
}
