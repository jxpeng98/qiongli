use std::io::{self, BufRead, Write};

use qiongli_runtime::protocol::{
    read_message as read_runtime_message, write_message as write_runtime_message,
};
pub use qiongli_runtime::protocol::{Framing, IncomingMessage};
use serde_json::Value;

pub fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<IncomingMessage>> {
    read_runtime_message(reader).map_err(qiongli_runtime::RuntimeError::into_io_error)
}

pub fn write_message<W: Write>(writer: &mut W, value: &Value, framing: Framing) -> io::Result<()> {
    write_runtime_message(writer, value, framing)
        .map_err(qiongli_runtime::RuntimeError::into_io_error)
}
