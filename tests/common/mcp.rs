//! A minimal MCP client for the integration tests. It starts the real
//! `zban mcp` and speaks the MCP stdio transport to it: JSON-RPC messages,
//! one per line, after the `initialize` handshake.

use std::io::{BufRead as _, BufReader, Write as _};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

use serde_json::{Value, json};

pub struct Client {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
    /// The result of `initialize`.
    pub initialized: Value,
}

/// The result of a tool call.
#[derive(Debug)]
pub struct Called {
    pub is_error: bool,
    /// The structured content.
    pub document: Value,
    /// The text content.
    pub text: String,
}

impl Client {
    /// Starts `zban [args] mcp` in `dir` and initializes the session.
    pub fn start(dir: &Path, args: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_zban"))
            .current_dir(dir)
            .args(args)
            .arg("mcp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("zban mcp starts");
        let stdin = child.stdin.take();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut client = Self {
            child,
            stdin,
            stdout,
            next_id: 0,
            initialized: Value::Null,
        };
        client.initialized = client.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": {"name": "zban-tests", "version": "0"},
            }),
        );
        client.send(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        client
    }

    /// The server's instructions.
    pub fn instructions(&self) -> &str {
        self.initialized["instructions"]
            .as_str()
            .expect("the server sends instructions")
    }

    fn send(&mut self, message: &Value) {
        let stdin = self.stdin.as_mut().unwrap();
        writeln!(stdin, "{message}").unwrap();
        stdin.flush().unwrap();
    }

    /// Sends a request and returns its response: `Ok` with the result, or
    /// `Err` with the JSON-RPC error.
    pub fn try_request(&mut self, method: &str, params: Value) -> Result<Value, Value> {
        self.next_id += 1;
        let id = self.next_id;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        loop {
            let mut line = String::new();
            let read = self.stdout.read_line(&mut line).unwrap();
            assert!(read > 0, "the server closed its output during {method}");
            let message: Value = serde_json::from_str(&line)
                .unwrap_or_else(|error| panic!("not JSON-RPC: {error}: {line:?}"));
            if message.get("method").is_some() || message["id"] != json!(id) {
                continue; // a notification or a request from the server
            }
            return match message.get("error") {
                Some(error) => Err(error.clone()),
                None => Ok(message["result"].clone()),
            };
        }
    }

    pub fn request(&mut self, method: &str, params: Value) -> Value {
        self.try_request(method, params)
            .unwrap_or_else(|error| panic!("{method} failed: {error}"))
    }

    /// The tools the server lists.
    pub fn tools(&mut self) -> Vec<Value> {
        let listed = self.request("tools/list", json!({}));
        assert!(listed.get("nextCursor").is_none(), "{listed}");
        listed["tools"].as_array().unwrap().clone()
    }

    /// Calls a tool and checks that its text content carries the same JSON
    /// document as its structured content.
    pub fn call(&mut self, tool: &str, arguments: Value) -> Called {
        let result = self.request("tools/call", json!({"name": tool, "arguments": arguments}));
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1, "{result}");
        assert_eq!(content[0]["type"], "text", "{result}");
        let text = content[0]["text"].as_str().unwrap().to_owned();
        let document = result["structuredContent"].clone();
        assert_eq!(
            serde_json::from_str::<Value>(&text).unwrap(),
            document,
            "{tool}: the text and the structured content differ"
        );
        Called {
            is_error: result["isError"] == json!(true),
            document,
            text,
        }
    }

    /// Closes the session and waits for the server to exit.
    pub fn finish(mut self) -> ExitStatus {
        self.stdin = None;
        self.child.wait().unwrap()
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.stdin = None;
        let _ = self.child.wait();
    }
}
