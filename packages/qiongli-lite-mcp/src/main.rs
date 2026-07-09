use std::io::{self, BufReader};

use qiongli_lite_mcp::mcp::protocol::{read_message, write_message};
use qiongli_lite_mcp::mcp::server::{McpRequest, McpServer};
use serde_json::json;

fn main() {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--transport" => {
                let transport = args.next().unwrap_or_else(|| "stdio".to_string());
                if transport != "stdio" {
                    eprintln!("unsupported transport: {transport}");
                    std::process::exit(2);
                }
            }
            _ => {}
        }
    }

    let server = McpServer::new("qiongli-literature-provider", env!("CARGO_PKG_VERSION"));
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());
    let mut stdout = io::stdout();

    loop {
        let message = match read_message(&mut reader) {
            Ok(Some(message)) => message,
            Ok(None) => break,
            Err(_) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": {"code": -32700, "message": "Parse error"}
                });
                let _ = write_message(
                    &mut stdout,
                    &response,
                    qiongli_lite_mcp::mcp::protocol::Framing::Line,
                );
                break;
            }
        };
        let response = match serde_json::from_str::<McpRequest>(&message.payload) {
            Ok(request) => {
                let is_notification = request.id.is_none();
                let response = server.handle(request);
                if is_notification {
                    continue;
                }
                response
            }
            Err(_) => json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {"code": -32700, "message": "Parse error"}
            }),
        };
        write_message(&mut stdout, &response, message.framing)
            .expect("failed to write MCP response");
    }
}
