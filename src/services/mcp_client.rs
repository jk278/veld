//! MCP (Model Context Protocol) client
//! 简洁的 MCP stdio 客户端实现

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, ChildStderr, Command, Stdio};
use std::thread;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// MCP tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// MCP client for stdio transport
pub struct McpClient {
    _child: Option<Child>,
    _stdin: Option<ChildStdin>,
    _stdout: Option<ChildStdout>,
    _stderr: Option<ChildStderr>,
    request_id: i64,
    initialized: bool,
}

impl McpClient {
    /// Create new MCP client by spawning server process
    pub fn connect(command: &str, args: &[String], env: Option<&std::collections::HashMap<String, String>>) -> Result<Self, String> {
        // On Windows, npx is npx.cmd (Command::new only searches for .exe)
        let actual_command = if cfg!(target_os = "windows") && command == "npx" {
            "npx.cmd"
        } else {
            command
        };

        let mut cmd = Command::new(actual_command);
        cmd.args(args);

        // Set environment variables if provided
        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // On Windows, hide the console window
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn MCP server ({}): {}", actual_command, e))?;

        let stdin = child.stdin.take().ok_or("Failed to get stdin".to_string())?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout".to_string())?;
        let stderr = child.stderr.take().ok_or("Failed to get stderr".to_string())?;

        // Spawn thread to log stderr output
        thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(msg) = line {
                    eprintln!("[MCP STDERR] {}", msg);
                }
            }
        });

        Ok(Self {
            _child: Some(child),
            _stdin: Some(stdin),
            _stdout: Some(stdout),
            _stderr: None, // Ownership transferred to thread
            request_id: 0,
            initialized: false,
        })
    }

    /// Write a message to stdin (line-delimited JSON format)
    fn write_message(&mut self, message: &str) -> Result<(), String> {
        if let Some(stdin) = &mut self._stdin {
            // Simple line-delimited JSON format
            let content = format!("{}\n", message);
            stdin.write_all(content.as_bytes())
                .map_err(|e| format!("Failed to write: {}", e))?;
            stdin.flush()
                .map_err(|e| format!("Failed to flush: {}", e))?;
            Ok(())
        } else {
            Err("stdin not available".to_string())
        }
    }

    /// Read a message from stdout (line-delimited JSON format)
    fn read_message(&mut self) -> Result<String, String> {
        if let Some(stdout) = &mut self._stdout {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();

            reader.read_line(&mut line)
                .map_err(|e| format!("Failed to read: {}", e))?;

            Ok(line.trim().to_string())
        } else {
            Err("stdout not available".to_string())
        }
    }

    /// Send JSON-RPC request and get response
    fn send_request(&mut self, method: &str, params: Option<Value>) -> Result<Value, String> {
        let id = self.request_id;
        self.request_id += 1;

        // Convert None to empty object {} for MCP compatibility
        let params_value = params.unwrap_or_else(|| json!({}));

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params_value
        });

        eprintln!("[MCP] Sending: {}", request);

        self.write_message(&request.to_string())?;

        let response_str = self.read_message()?;
        eprintln!("[MCP] Received: {}", response_str);

        let response: Value = serde_json::from_str(&response_str)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // Check for error
        if let Some(err) = response.get("error") {
            return Err(format!("MCP error: {}", err));
        }

        Ok(response["result"].clone())
    }

    /// Send notification (no response expected)
    fn send_notification(&mut self, method: &str, params: Option<Value>) -> Result<(), String> {
        // Convert None to empty object {} for MCP compatibility
        let params_value = params.unwrap_or_else(|| json!({}));

        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params_value
        });

        eprintln!("[MCP] Sending notification: {}", notification);
        self.write_message(&notification.to_string())
    }

    /// Initialize MCP session (must be called first)
    fn initialize(&mut self) -> Result<(), String> {
        // Send initialize request
        self.send_request("initialize", Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "veld",
                "version": "0.1.0"
            }
        })))?;

        // Send initialized notification (REQUIRED by MCP spec)
        self.send_notification("notifications/initialized", None)?;

        self.initialized = true;
        Ok(())
    }

    /// List available tools
    pub fn list_tools(&mut self) -> Result<Vec<McpTool>, String> {
        if !self.initialized {
            self.initialize()?;
        }

        let result = self.send_request("tools/list", None)?;

        let tools = result["tools"]
            .as_array()
            .ok_or("Invalid tools response")?
            .iter()
            .map(|t| McpTool {
                name: t["name"].as_str().unwrap_or("").to_string(),
                description: t["description"].as_str().unwrap_or("").to_string(),
                input_schema: t["inputSchema"].clone(),
            })
            .collect();

        Ok(tools)
    }

    /// Call a tool
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<Value, String> {
        if !self.initialized {
            self.initialize()?;
        }

        let result = self.send_request("tools/call", Some(json!({
            "name": name,
            "arguments": arguments
        })))?;

        Ok(result)
    }

    /// Close the client connection
    pub fn close(&mut self) {
        self._child.take();
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        self.close();
    }
}
