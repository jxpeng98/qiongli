use std::io::{self, BufRead, Write};

use serde_json::Value;

const CONTENT_LENGTH: &str = "content-length";
const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
const MAX_HEADER_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    Line,
    ContentLength,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingMessage {
    pub payload: String,
    pub framing: Framing,
}

pub fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<IncomingMessage>> {
    loop {
        let mut first_line = String::new();
        if read_bounded_line(reader, &mut first_line, MAX_MESSAGE_BYTES)? == 0 {
            return Ok(None);
        }
        let first_line = first_line.trim_end_matches(['\r', '\n']);
        if first_line.trim().is_empty() {
            continue;
        }

        if let Some((name, value)) = first_line.split_once(':') {
            if name.trim().eq_ignore_ascii_case(CONTENT_LENGTH) {
                let length = value.trim().parse::<usize>().map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "invalid Content-Length")
                })?;
                if length > MAX_MESSAGE_BYTES {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "MCP message exceeds size limit",
                    ));
                }

                let mut header_bytes = first_line.len();
                loop {
                    let remaining =
                        MAX_HEADER_BYTES.checked_sub(header_bytes).ok_or_else(|| {
                            io::Error::new(
                                io::ErrorKind::InvalidData,
                                "MCP headers exceed size limit",
                            )
                        })?;
                    if remaining == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "MCP headers exceed size limit",
                        ));
                    }
                    let mut header = String::new();
                    if read_bounded_line(reader, &mut header, remaining)? == 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::UnexpectedEof,
                            "incomplete MCP headers",
                        ));
                    }
                    header_bytes += header.len();
                    if header == "\n" || header == "\r\n" {
                        break;
                    }
                }

                let mut payload = vec![0_u8; length];
                reader.read_exact(&mut payload)?;
                let payload = String::from_utf8(payload).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "MCP payload is not UTF-8")
                })?;
                return Ok(Some(IncomingMessage {
                    payload,
                    framing: Framing::ContentLength,
                }));
            }
        }

        if first_line.len() > MAX_MESSAGE_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MCP message exceeds size limit",
            ));
        }
        return Ok(Some(IncomingMessage {
            payload: first_line.to_string(),
            framing: Framing::Line,
        }));
    }
}

fn read_bounded_line<R: BufRead>(
    reader: &mut R,
    output: &mut String,
    maximum: usize,
) -> io::Result<usize> {
    let mut bytes = Vec::new();
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            break;
        }
        let count = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |position| position + 1);
        if bytes.len().saturating_add(count) > maximum {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "MCP line exceeds size limit",
            ));
        }
        bytes.extend_from_slice(&available[..count]);
        reader.consume(count);
        if bytes.last() == Some(&b'\n') {
            break;
        }
    }
    *output = String::from_utf8(bytes)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "MCP line is not UTF-8"))?;
    Ok(output.len())
}

pub fn write_message<W: Write>(writer: &mut W, value: &Value, framing: Framing) -> io::Result<()> {
    let payload = serde_json::to_vec(value).map_err(io::Error::other)?;
    match framing {
        Framing::Line => {
            writer.write_all(&payload)?;
            writer.write_all(b"\n")?;
        }
        Framing::ContentLength => {
            write!(writer, "Content-Length: {}\r\n\r\n", payload.len())?;
            writer.write_all(&payload)?;
        }
    }
    writer.flush()
}
