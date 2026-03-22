use reqwest::ClientBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Deserialize, Debug, Clone)]
struct Config {
    #[serde(default = "default_url")]
    lm_studio_url: String,
    model: String,
    #[serde(default = "default_request_timeout_secs")]
    request_timeout_secs: u64,
    #[serde(default = "default_max_completion_tokens")]
    max_completion_tokens: u32,
    #[serde(default)]
    stop_sequences: Vec<String>,
    prompt_templates: PromptTemplates,
}

#[derive(Deserialize, Debug, Clone)]
struct PromptTemplates {
    generate_code: String,
    edit_code: String,
    complete_code: String,
    explain_code: String,
}

fn default_url() -> String {
    "http://localhost:1234".to_string()
}

fn default_request_timeout_secs() -> u64 {
    110
}

fn default_max_completion_tokens() -> u32 {
    1200
}

// JSON-RPC Requests
#[derive(Deserialize, Debug)]
struct RpcRequest {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

// JSON-RPC Responses
#[derive(Serialize, Debug)]
struct RpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Serialize, Debug)]
struct RpcError {
    code: i32,
    message: String,
}

fn extract_response_text(body: &Value) -> Option<&str> {
    body.get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|content| !content.is_empty())
}

async fn check_for_updates() {
    let client = reqwest::Client::builder()
        .user_agent("lm-bridge-updater")
        .timeout(Duration::from_secs(4))
        .build();
    let Ok(client) = client else { return; };

    let resp = client.get("https://api.github.com/repos/psipher/lm-bridge/releases/latest").send().await;
    if let Ok(resp) = resp {
        if let Ok(json) = resp.json::<Value>().await {
            if let Some(tag_name) = json.get("tag_name").and_then(|v| v.as_str()) {
                let latest_version = tag_name.trim_start_matches('v');
                let current_version = env!("CARGO_PKG_VERSION");
                if latest_version != current_version && !latest_version.is_empty() {
                    eprintln!(
                        "\n⚠️ [UPDATE AVAILABLE] lm-bridge v{} is out! (You are running v{})",
                        latest_version, current_version
                    );
                    if let Some(url) = json.get("html_url").and_then(|v| v.as_str()) {
                        eprintln!("Download at: {}\n", url);
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Resolve the configuration file path safely by checking up to three parent directories.
    let mut config_dir = env::current_exe()?.parent().map(|p| p.to_path_buf());
    let mut resolved_config = None;

    // Resolve config.toml by walking up from the exe directory.
    // For target/release/lm-bridge.exe this checks:
    //   iter 0: target/release/config.toml  (exe dir)
    //   iter 1: target/config.toml          (parent)
    //   iter 2: <project_root>/config.toml  (grandparent) ← found here
    // NOTE: The `ref dir` borrow on config_dir is scoped to the `if let` block
    // and is released before the `.and_then()` move on the next line. This is
    // intentional and verified correct by the borrow checker.
    for _ in 0..=2 {
        if let Some(ref dir) = config_dir {
            let candidate = dir.join("config.toml");
            if candidate.exists() {
                resolved_config = Some(candidate);
                break;
            }
        }
        config_dir = config_dir.and_then(|d| d.parent().map(|p| p.to_path_buf()));
    }

    let config_path = resolved_config.unwrap_or_else(|| {
        // Zero-setup bootstrapping: Auto-generate config.toml if missing
        let exe_dir = env::current_exe().unwrap().parent().unwrap().to_path_buf();
        let path = exe_dir.join("config.toml");
        let default_config = include_str!("../config.toml");
        let _ = std::fs::write(&path, default_config);
        path
    });

    eprintln!("Using config at: {}", config_path.display());

    let config_content = std::fs::read_to_string(&config_path).unwrap_or_else(|e| {
        eprintln!("Failed to read {}: {}", config_path.display(), e);
        std::process::exit(1);
    });

    let mut config: Config = toml::from_str(&config_content).unwrap_or_else(|e| {
        eprintln!("Failed to parse config.toml: {}", e);
        std::process::exit(1);
    });

    if let Ok(env_model) = env::var("LM_STUDIO_MODEL") {
        config.model = env_model;
    }

    eprintln!("Active model: {}", config.model);

    // Run the update checker in the background so it doesn't block startup
    tokio::spawn(check_for_updates());

    // Generate mcp_registration.json snippet for the user
    if let Some(root) = config_path.parent() {
        let snippet_path = root.join("mcp_registration.json");
        // Ensure path for JSON just uses normal string for generic path serialization
        let exe_path = env::current_exe()?.to_string_lossy().to_string();
        let snippet = json!({
            "mcpServers": {
                "local_llm": {
                    "command": exe_path,
                    "args": [],
                    "env": {
                        "LM_STUDIO_MODEL": config.model
                    }
                }
            }
        });
        if let Ok(content) = serde_json::to_string_pretty(&snippet) {
            let _ = std::fs::write(&snippet_path, content);
        }
    }

    // Check for interactive (double-click) mode or --register flag
    let args: Vec<String> = env::args().collect();
    use std::io::IsTerminal;
    let is_interactive = std::io::stdin().is_terminal();

    if args.contains(&"--register".to_string()) || is_interactive {
        println!("\n✅ Successfully generated mcp_registration.json and config.toml in this directory.");
        if is_interactive {
            println!("MCP Setup is complete! Please attach this executable to your AI client.");
            println!("\nPress Enter to close this window...");
            let mut buf = String::new();
            let _ = std::io::stdin().read_line(&mut buf);
        }
        return Ok(());
    }

    let client = ClientBuilder::new()
        .timeout(Duration::from_secs(config.request_timeout_secs))
        .build()?;

    let stdin = tokio::io::stdin();
    let mut reader = BufReader::new(stdin).lines();

    while let Ok(Some(line)) = reader.next_line().await {
        if line.trim().is_empty() {
            continue;
        }

        let request: Result<RpcRequest, _> = serde_json::from_str(&line);
        let req = match request {
            Ok(r) => r,
            Err(e) => {
                let err_resp = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                match serde_json::to_string(&err_resp) {
                    Ok(s) => println!("{}", s),
                    Err(e) => eprintln!("Serialization error: {}", e),
                }
                continue;
            }
        };

        if req.method == "notifications/initialized" || req.method == "notifications/cancelled" {
            continue; // Protocol notifications — no response required
        }

        let id = match req.id {
            Some(i) => i,
            None => continue, // Ignore JSON-RPC notifications entirely (no ID means no response needed)
        };

        if req.method == "initialize" {
            let resp = RpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05", // Required by standard MCP
                    "capabilities": {
                        "tools": { "listChanged": false }
                    },
                    "serverInfo": {
                        "name": "local_llm",
                        "version": "0.2.1"
                    }
                })),
                error: None,
            };
            match serde_json::to_string(&resp) {
                Ok(s) => println!("{}", s),
                Err(e) => eprintln!("Serialization error: {}", e),
            }
            continue;
        }

        if req.method == "tools/list" {
            let resp = RpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(json!({
                    "tools": [
                        {
                            "name": "local_generate",
                            "description": "sends a full coding task, returns generated code.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "task": { "type": "string" },
                                    "language": { "type": "string" },
                                    "context": { "type": "string" },
                                    "file_path": { "type": "string" }
                                },
                                "required": ["task", "language", "context", "file_path"]
                            }
                        },
                        {
                            "name": "local_edit",
                            "description": "sends existing code + edit instruction, returns modified code.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "existing_code": { "type": "string" },
                                    "instruction": { "type": "string" },
                                    "language": { "type": "string" }
                                },
                                "required": ["existing_code", "instruction", "language"]
                            }
                        },
                        {
                            "name": "local_complete",
                            "description": "fills in a partial snippet.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "prefix": { "type": "string" },
                                    "language": { "type": "string" },
                                    "context": { "type": "string" }
                                },
                                "required": ["prefix", "language", "context"]
                            }
                        },
                        {
                            "name": "local_explain",
                            "description": "returns a plain-language explanation.",
                            "inputSchema": {
                                "type": "object",
                                "properties": {
                                    "code": { "type": "string" },
                                    "language": { "type": "string" }
                                },
                                "required": ["code", "language"]
                            }
                        }
                    ]
                })),
                error: None,
            };
            match serde_json::to_string(&resp) {
                Ok(s) => println!("{}", s),
                Err(e) => eprintln!("Serialization error: {}", e),
            }
            continue;
        }

        if req.method == "tools/call" {
            let params = req.params;
            let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = params.get("arguments").unwrap_or(&Value::Null);

            eprintln!("Tool called: {}", name);

            let prompt = match name {
                "local_generate" => {
                    let task = args.get("task").and_then(|v| v.as_str()).unwrap_or("");
                    let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("");
                    let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");
                    let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
                    config
                        .prompt_templates
                        .generate_code
                        .replace("{task}", task)
                        .replace("{language}", language)
                        .replace("{context}", context)
                        .replace("{file_path}", file_path)
                }
                "local_edit" => {
                    let existing_code = args
                        .get("existing_code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let instruction = args
                        .get("instruction")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("");
                    config
                        .prompt_templates
                        .edit_code
                        .replace("{existing_code}", existing_code)
                        .replace("{instruction}", instruction)
                        .replace("{language}", language)
                }
                "local_complete" => {
                    let prefix = args.get("prefix").and_then(|v| v.as_str()).unwrap_or("");
                    let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("");
                    let context = args.get("context").and_then(|v| v.as_str()).unwrap_or("");
                    config
                        .prompt_templates
                        .complete_code
                        .replace("{prefix}", prefix)
                        .replace("{language}", language)
                        .replace("{context}", context)
                }
                "local_explain" => {
                    let code = args.get("code").and_then(|v| v.as_str()).unwrap_or("");
                    let language = args.get("language").and_then(|v| v.as_str()).unwrap_or("");
                    config
                        .prompt_templates
                        .explain_code
                        .replace("{code}", code)
                        .replace("{language}", language)
                }
                _ => {
                    let resp = RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: id.clone(),
                        result: None,
                        error: Some(RpcError {
                            code: -32601,
                            message: format!("Unknown tool: {}", name),
                        }),
                    };
                    match serde_json::to_string(&resp) {
                        Ok(s) => println!("{}", s),
                        Err(e) => eprintln!("Serialization error: {}", e),
                    }
                    continue;
                }
            };

            // Call LM Studio
            let mut payload = serde_json::Map::new();
            payload.insert("model".to_string(), Value::String(config.model.clone()));
            payload.insert(
                "messages".to_string(),
                json!([
                    { "role": "user", "content": prompt }
                ]),
            );
            payload.insert(
                "max_tokens".to_string(),
                Value::Number(config.max_completion_tokens.into()),
            );

            if !config.stop_sequences.is_empty() {
                payload.insert("stop".to_string(), json!(config.stop_sequences));
            }

            let url = format!("{}/v1/chat/completions", config.lm_studio_url);
            let res = client.post(&url).json(&Value::Object(payload)).send().await;

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        let body: Value = response.json().await.unwrap_or(Value::Null);
                        let Some(content) = extract_response_text(&body) else {
                            let resp = RpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: id.clone(),
                                result: Some(json!({
                                    "isError": true,
                                    "content": [
                                        { "type": "text", "text": format!("LM Studio returned an empty completion for tool {}", name) }
                                    ]
                                })),
                                error: None,
                            };
                            match serde_json::to_string(&resp) {
                                Ok(s) => println!("{}", s),
                                Err(e) => eprintln!("Serialization error: {}", e),
                            }
                            continue;
                        };
                        let resp = RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: id.clone(),
                            result: Some(json!({
                                "content": [
                                    { "type": "text", "text": content }
                                ]
                            })),
                            error: None,
                        };
                        match serde_json::to_string(&resp) {
                            Ok(s) => println!("{}", s),
                            Err(e) => eprintln!("Serialization error: {}", e),
                        }
                    } else {
                        let status = response.status();
                        let error_text = response.text().await.unwrap_or_default();
                        let resp = RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: id.clone(),
                            result: Some(json!({
                                "isError": true,
                                "content": [
                                    { "type": "text", "text": format!("HTTP Error {}: {}", status, error_text) }
                                ]
                            })),
                            error: None,
                        };
                        match serde_json::to_string(&resp) {
                            Ok(s) => println!("{}", s),
                            Err(e) => eprintln!("Serialization error: {}", e),
                        }
                    }
                }
                Err(e) => {
                    let resp = RpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: id.clone(),
                        result: Some(json!({
                            "isError": true,
                            "content": [
                                { "type": "text", "text": format!("LM Studio connection failed: {}", e) }
                            ]
                        })),
                        error: None,
                    };
                    match serde_json::to_string(&resp) {
                        Ok(s) => println!("{}", s),
                        Err(e) => eprintln!("Serialization error: {}", e),
                    }
                }
            }
            continue;
        }

        // Unknown method
        let resp = RpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: format!("Method not found: {}", req.method),
            }),
        };
        match serde_json::to_string(&resp) {
            Ok(s) => println!("{}", s),
            Err(e) => eprintln!("Serialization error: {}", e),
        }
    }

    Ok(())
}
