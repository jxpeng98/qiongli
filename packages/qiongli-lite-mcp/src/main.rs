use std::io::{self, BufRead, Write};

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
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let Ok(line) = line else {
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<McpRequest>(&line) {
            Ok(request) => server.handle(request),
            Err(_) => json!({
                "jsonrpc": "2.0",
                "id": null,
                "error": {"code": -32700, "message": "Parse error"}
            }),
        };
        writeln!(stdout, "{response}").expect("failed to write MCP response");
        stdout.flush().expect("failed to flush MCP response");
    }
}
