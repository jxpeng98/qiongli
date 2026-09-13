use std::io::{BufReader, Cursor, Read, Write};
use std::process::{Command, Stdio};

use qiongli_lite_mcp::mcp::protocol::{read_message, write_message, Framing};
use serde_json::json;

#[test]
fn reads_newline_delimited_and_raw_eof_messages() {
    let input = b"{\"jsonrpc\":\"2.0\",\"id\":1}\n{\"jsonrpc\":\"2.0\",\"id\":2}";
    let mut reader = BufReader::new(Cursor::new(input));

    let first = read_message(&mut reader).unwrap().unwrap();
    let second = read_message(&mut reader).unwrap().unwrap();

    assert_eq!(first.framing, Framing::Line);
    assert_eq!(second.framing, Framing::Line);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&first.payload).unwrap()["id"],
        1
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&second.payload).unwrap()["id"],
        2
    );
    assert!(read_message(&mut reader).unwrap().is_none());
}

#[test]
fn reads_content_length_using_utf8_byte_length() {
    let payload = "{\"jsonrpc\":\"2.0\",\"query\":\"治理\"}";
    let input = format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload);
    let mut reader = BufReader::new(Cursor::new(input.into_bytes()));

    let message = read_message(&mut reader).unwrap().unwrap();

    assert_eq!(message.framing, Framing::ContentLength);
    assert_eq!(message.payload, payload);
}

#[test]
fn writes_responses_using_the_request_framing() {
    let response = json!({"jsonrpc": "2.0", "id": 1, "result": {"title": "治理"}});
    let mut framed = Vec::new();
    let mut line = Vec::new();

    write_message(&mut framed, &response, Framing::ContentLength).unwrap();
    write_message(&mut line, &response, Framing::Line).unwrap();

    let separator = framed
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .unwrap();
    let header = std::str::from_utf8(&framed[..separator]).unwrap();
    let declared = header
        .strip_prefix("Content-Length: ")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    assert_eq!(declared, framed.len() - separator - 4);
    assert_eq!(line.last(), Some(&b'\n'));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&line).unwrap(),
        response
    );
}

#[test]
fn rejects_oversized_unterminated_line_without_unbounded_read_line() {
    let input = vec![b'x'; 8 * 1024 * 1024 + 1];
    let mut reader = BufReader::new(Cursor::new(input));

    let error = read_message(&mut reader).unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn stdio_notifications_do_not_produce_json_rpc_responses() {
    let binary = env!("CARGO_BIN_EXE_qiongli-literature-provider");
    let mut child = Command::new(binary)
        .arg("--transport")
        .arg("stdio")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(
            b"{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\",\"params\":{}}\n\
              {\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\",\"params\":{}}\n",
        )
        .unwrap();
    drop(stdin);
    let mut stdout = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut stdout)
        .unwrap();
    assert!(child.wait().unwrap().success());

    let responses: Vec<serde_json::Value> = stdout
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], 1);
}
