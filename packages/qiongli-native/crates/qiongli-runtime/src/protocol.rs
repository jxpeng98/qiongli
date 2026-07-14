use std::io::{BufRead, Write};

use serde::Serialize;

use crate::{RuntimeError, RuntimeErrorCode};

const CONTENT_LENGTH: &str = "content-length";
pub const MAX_MCP_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_MCP_HEADER_BYTES: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Framing {
    Line,
    ContentLength,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IncomingMessage {
    pub payload: String,
    pub framing: Framing,
}

pub fn read_message<R: BufRead>(reader: &mut R) -> Result<Option<IncomingMessage>, RuntimeError> {
    loop {
        let mut first_line = String::new();
        if read_bounded_line(
            reader,
            &mut first_line,
            MAX_MCP_MESSAGE_BYTES,
            RuntimeErrorCode::McpMessageTooLarge,
        )? == 0
        {
            return Ok(None);
        }
        let raw_first_line_bytes = first_line.len();
        let first_line = first_line.trim_end_matches(['\r', '\n']);
        if first_line.trim().is_empty() {
            continue;
        }

        if let Some((name, value)) = first_line.split_once(':')
            && name.trim().eq_ignore_ascii_case(CONTENT_LENGTH)
        {
            if raw_first_line_bytes > MAX_MCP_HEADER_BYTES {
                return Err(RuntimeError::new(RuntimeErrorCode::McpHeadersTooLarge));
            }
            let length = parse_content_length(value)?;
            if length > MAX_MCP_MESSAGE_BYTES {
                return Err(RuntimeError::new(RuntimeErrorCode::McpMessageTooLarge));
            }
            read_headers(reader, raw_first_line_bytes)?;

            let mut payload = vec![0_u8; length];
            if let Err(error) = reader.read_exact(&mut payload) {
                return if error.kind() == std::io::ErrorKind::UnexpectedEof {
                    Err(RuntimeError::new(RuntimeErrorCode::IncompleteMcpMessage))
                } else {
                    Err(RuntimeError::input_failed(error.kind()))
                };
            }
            let payload = String::from_utf8(payload)
                .map_err(|_| RuntimeError::new(RuntimeErrorCode::McpPayloadNotUtf8))?;
            return Ok(Some(IncomingMessage {
                payload,
                framing: Framing::ContentLength,
            }));
        }

        return Ok(Some(IncomingMessage {
            payload: first_line.to_string(),
            framing: Framing::Line,
        }));
    }
}

fn parse_content_length(value: &str) -> Result<usize, RuntimeError> {
    let value = value.trim();
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(RuntimeError::new(RuntimeErrorCode::InvalidMcpMessage));
    }
    value
        .parse::<usize>()
        .map_err(|_| RuntimeError::new(RuntimeErrorCode::InvalidMcpMessage))
}

fn read_headers<R: BufRead>(reader: &mut R, first_line_bytes: usize) -> Result<(), RuntimeError> {
    if first_line_bytes > MAX_MCP_HEADER_BYTES {
        return Err(RuntimeError::new(RuntimeErrorCode::McpHeadersTooLarge));
    }
    let mut header_bytes = first_line_bytes;
    loop {
        let remaining = MAX_MCP_HEADER_BYTES
            .checked_sub(header_bytes)
            .filter(|remaining| *remaining > 0)
            .ok_or_else(|| RuntimeError::new(RuntimeErrorCode::McpHeadersTooLarge))?;
        let mut header = String::new();
        if read_bounded_line(
            reader,
            &mut header,
            remaining,
            RuntimeErrorCode::McpHeadersTooLarge,
        )? == 0
        {
            return Err(RuntimeError::new(RuntimeErrorCode::IncompleteMcpMessage));
        }
        header_bytes += header.len();
        if header == "\n" || header == "\r\n" {
            return Ok(());
        }
    }
}

fn read_bounded_line<R: BufRead>(
    reader: &mut R,
    output: &mut String,
    maximum: usize,
    too_large: RuntimeErrorCode,
) -> Result<usize, RuntimeError> {
    let mut bytes = Vec::new();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|error| RuntimeError::input_failed(error.kind()))?;
        if available.is_empty() {
            break;
        }
        let count = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |position| position + 1);
        if bytes.len().saturating_add(count) > maximum {
            return Err(RuntimeError::new(too_large));
        }
        bytes.extend_from_slice(&available[..count]);
        reader.consume(count);
        if bytes.last() == Some(&b'\n') {
            break;
        }
    }
    *output = String::from_utf8(bytes)
        .map_err(|_| RuntimeError::new(RuntimeErrorCode::McpPayloadNotUtf8))?;
    Ok(output.len())
}

pub fn write_message<W: Write, T: Serialize>(
    writer: &mut W,
    value: &T,
    framing: Framing,
) -> Result<(), RuntimeError> {
    let payload = serde_json::to_vec(value)
        .map_err(|_| RuntimeError::new(RuntimeErrorCode::McpOutputSerializationFailed))?;
    if payload.len() > MAX_MCP_MESSAGE_BYTES {
        return Err(RuntimeError::new(RuntimeErrorCode::McpMessageTooLarge));
    }
    match framing {
        Framing::Line => {
            write_all(writer, &payload)?;
            write_all(writer, b"\n")?;
        }
        Framing::ContentLength => {
            let header = format!("Content-Length: {}\r\n\r\n", payload.len());
            write_all(writer, header.as_bytes())?;
            write_all(writer, &payload)?;
        }
    }
    writer
        .flush()
        .map_err(|error| RuntimeError::output_failed(error.kind()))
}

fn write_all<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<(), RuntimeError> {
    writer
        .write_all(bytes)
        .map_err(|error| RuntimeError::output_failed(error.kind()))
}

#[cfg(test)]
mod tests {
    use std::io::{self, BufReader, Cursor, Read};

    use serde::ser::{Error as _, Serializer};
    use serde_json::{Value, json};

    use super::*;

    const CANARY: &str = "private-framing-canary";

    #[test]
    fn reads_lines_blank_prefixes_and_raw_eof() {
        let input = b"\n \r\n{\"id\":1}\n{\"id\":2}";
        let mut reader = BufReader::new(Cursor::new(input));

        let first = read_message(&mut reader).unwrap().unwrap();
        let second = read_message(&mut reader).unwrap().unwrap();

        assert_eq!(first.framing, Framing::Line);
        assert_eq!(first.payload, r#"{"id":1}"#);
        assert_eq!(second.payload, r#"{"id":2}"#);
        assert!(read_message(&mut reader).unwrap().is_none());
    }

    #[test]
    fn reads_and_writes_content_length_using_utf8_bytes() {
        let payload = r#"{"query":"治理"}"#;
        let input = format!(
            "cOnTeNt-LeNgTh: {}\r\nX-Test: yes\r\n\r\n{payload}",
            payload.len()
        );
        let mut reader = BufReader::new(Cursor::new(input.into_bytes()));
        let message = read_message(&mut reader).unwrap().unwrap();
        assert_eq!(message.framing, Framing::ContentLength);
        assert_eq!(message.payload, payload);

        let response = json!({"title": "治理"});
        let mut output = Vec::new();
        write_message(&mut output, &response, Framing::ContentLength).unwrap();
        let separator = output
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .unwrap();
        let declared = std::str::from_utf8(&output[..separator])
            .unwrap()
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse::<usize>()
            .unwrap();
        assert_eq!(declared, output.len() - separator - 4);
    }

    #[test]
    fn rejects_invalid_lengths_and_redacts_peer_input() {
        let input = format!("Content-Length: {CANARY}\r\n\r\n");
        let mut reader = BufReader::new(Cursor::new(input.into_bytes()));
        let error = read_message(&mut reader).unwrap_err();
        assert_eq!(error.code(), RuntimeErrorCode::InvalidMcpMessage);
        assert!(!error.to_string().contains(CANARY));
    }

    #[test]
    fn rejects_message_and_header_overflow() {
        let line = vec![b'x'; MAX_MCP_MESSAGE_BYTES + 1];
        let mut line_reader = BufReader::new(Cursor::new(line));
        assert_eq!(
            read_message(&mut line_reader).unwrap_err().code(),
            RuntimeErrorCode::McpMessageTooLarge
        );

        let declared = format!("Content-Length: {}\r\n\r\n", MAX_MCP_MESSAGE_BYTES + 1);
        let mut declared_reader = BufReader::new(Cursor::new(declared.into_bytes()));
        assert_eq!(
            read_message(&mut declared_reader).unwrap_err().code(),
            RuntimeErrorCode::McpMessageTooLarge
        );

        let mut headers = b"Content-Length: 0\r\nX-Test: ".to_vec();
        headers.extend(std::iter::repeat_n(
            b'x',
            MAX_MCP_HEADER_BYTES - headers.len() + 1,
        ));
        headers.extend_from_slice(b"\r\n\r\n");
        let mut header_reader = BufReader::new(Cursor::new(headers));
        assert_eq!(
            read_message(&mut header_reader).unwrap_err().code(),
            RuntimeErrorCode::McpHeadersTooLarge
        );
    }

    #[test]
    fn rejects_incomplete_or_non_utf8_messages() {
        let mut incomplete_headers =
            BufReader::new(Cursor::new(b"Content-Length: 1\r\n".as_slice()));
        assert_eq!(
            read_message(&mut incomplete_headers).unwrap_err().code(),
            RuntimeErrorCode::IncompleteMcpMessage
        );

        let mut incomplete_payload =
            BufReader::new(Cursor::new(b"Content-Length: 2\r\n\r\nx".as_slice()));
        assert_eq!(
            read_message(&mut incomplete_payload).unwrap_err().code(),
            RuntimeErrorCode::IncompleteMcpMessage
        );

        let mut invalid_utf8 = BufReader::new(Cursor::new([0xff, b'\n']));
        assert_eq!(
            read_message(&mut invalid_utf8).unwrap_err().code(),
            RuntimeErrorCode::McpPayloadNotUtf8
        );
    }

    #[test]
    fn classifies_input_output_and_serialization_failures_without_sources() {
        let mut input = BufReader::new(FailingReader);
        let input_error = read_message(&mut input).unwrap_err();
        assert_eq!(input_error.code(), RuntimeErrorCode::McpInputFailed);
        assert_eq!(input_error.io_kind(), Some(io::ErrorKind::PermissionDenied));

        let mut output = FailingWriter;
        let output_error = write_message(&mut output, &json!({}), Framing::Line).unwrap_err();
        assert_eq!(output_error.code(), RuntimeErrorCode::McpOutputFailed);
        assert_eq!(output_error.io_kind(), Some(io::ErrorKind::BrokenPipe));

        let serialization_error =
            write_message(&mut Vec::new(), &FailingSerialize, Framing::Line).unwrap_err();
        assert_eq!(
            serialization_error.code(),
            RuntimeErrorCode::McpOutputSerializationFailed
        );
        for error in [input_error, output_error, serialization_error] {
            assert!(!error.to_string().contains(CANARY));
        }
    }

    #[test]
    fn writes_line_framing_and_enforces_output_limit() {
        let response = json!({"id": 1});
        let mut line = Vec::new();
        write_message(&mut line, &response, Framing::Line).unwrap();
        assert_eq!(line.last(), Some(&b'\n'));
        assert_eq!(serde_json::from_slice::<Value>(&line).unwrap(), response);

        let oversized = "x".repeat(MAX_MCP_MESSAGE_BYTES);
        let error = write_message(&mut Vec::new(), &oversized, Framing::Line).unwrap_err();
        assert_eq!(error.code(), RuntimeErrorCode::McpMessageTooLarge);
    }

    struct FailingReader;

    impl Read for FailingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, CANARY))
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, CANARY))
        }

        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, CANARY))
        }
    }

    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(S::Error::custom(CANARY))
        }
    }
}
