// src/installer.rs

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

/// Interactive installer for the Gemini integration.
pub async fn run_interactive_installer(
    exe_path: String,
    model: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Resolve home directory and gemini paths
    #[cfg(windows)]
    let home_dir = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
    #[cfg(not(windows))]
    let home_dir = env::var("HOME").unwrap_or_else(|_| "/".to_string());

    let gemini_dir = PathBuf::from(home_dir).join(".gemini");
    let mcp_config_path = gemini_dir.join("antigravity/mcp_config.json");
    let gemini_md_path = gemini_dir.join("GEMINI.md");

    // Helper to read a line from stdin
    fn read_line(prompt: &str) -> io::Result<String> {
        print!("{}", prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(input.trim().to_string())
    }

    println!("Select an option:");
    println!("[1] Setup Manually (Generate files locally)");
    println!("[2] Auto-Install for Google Antigravity");
    let choice = read_line("Enter your choice: ")?;

    match choice.as_str() {
        "1" => {
            println!("\n✅ Successfully generated mcp_registration.json and config.toml in this directory.");
            println!("MCP Setup is complete! Please attach this executable to your AI client.");
            return Ok(()); // Let main.rs handle local write
        }
        "2" => {
            if !gemini_dir.exists() {
                println!("Unable to find Antigravity configuration folder at {}. Falling back to manual mode.", gemini_dir.display());
                return Ok(());
            }

            println!("\nWhat would you like to install?");
            println!("[1] MCP Config Only");
            println!("[2] Agent Rules Only");
            println!("[3] Both");
            let sub_choice = read_line("Enter your choice: ")?;

            println!(
                "\nThe following files will be modified:\n- {}\n- {}",
                mcp_config_path.display(),
                gemini_md_path.display()
            );
            println!("Note: Existing agent rules may be commented out.");
            let confirm = read_line("Proceed? (y/n): ")?;
            if !confirm.eq_ignore_ascii_case("y") {
                println!("Installation aborted.");
                return Ok(());
            }

            // MCP Config
            if sub_choice == "1" || sub_choice == "3" {
                let mut config: Value = if mcp_config_path.exists() {
                    let data = fs::read_to_string(&mcp_config_path)?;
                    match serde_json::from_str(&data) {
                        Ok(v) => v,
                        Err(e) => {
                            eprintln!("Failed to parse existing MCP config: {}. Please fix it manually. Aborting auto-install.", e);
                            return Ok(());
                        }
                    }
                } else {
                    json!({"mcpServers": {}})
                };

                let local_node = json!({
                    "command": exe_path,
                    "args": [],
                    "env": { "LM_STUDIO_MODEL": model }
                });

                if let Some(obj) = config.as_object_mut() {
                    if !obj.contains_key("mcpServers") {
                        obj.insert("mcpServers".to_string(), json!({}));
                    }
                    if let Some(servers) = obj.get_mut("mcpServers").and_then(|v| v.as_object_mut())
                    {
                        servers.insert("local_llm".to_string(), local_node);
                        let new_json = serde_json::to_string_pretty(&config)?;
                        fs::write(&mcp_config_path, new_json)?;
                        println!("Updated MCP config at {}", mcp_config_path.display());
                    }
                }
            }

            // GEMINI.md
            if sub_choice == "2" || sub_choice == "3" {
                let mut content = String::new();
                if gemini_md_path.exists() {
                    content = fs::read_to_string(&gemini_md_path)?;
                }

                // Append the integration snippet
                let include_snippet = include_str!("../integrations/google_antigravity/GEMINI.md");
                if !content.contains("Privacy & Tool Routing") {
                    content.push_str("\n\n");
                    content.push_str(include_snippet);
                    fs::write(&gemini_md_path, content)?;
                    println!("Updated GEMINI.md at {}", gemini_md_path.display());
                } else {
                    println!("Local LLM rules already exist in GEMINI.md. Skipping.");
                }
            }
        }
        _ => {
            eprintln!("Invalid choice. Exiting.");
            return Ok(());
        }
    }

    Ok(())
}
