use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::path::PathBuf;

use serde_json::{json, Value};
use toml_edit::{value, Array, DocumentMut, Item, Table};

const CODEX_STARTUP_TIMEOUT_SECS: i64 = 20;
const CODEX_TOOL_TIMEOUT_SECS: i64 = 120;

fn read_line(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn codex_skill_contents() -> &'static str {
    include_str!("../integrations/codex/SKILL.md")
}

fn antigravity_rules_contents() -> &'static str {
    include_str!("../integrations/google_antigravity/GEMINI.md")
}

#[cfg(windows)]
fn home_dir() -> PathBuf {
    PathBuf::from(env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string()))
}

#[cfg(not(windows))]
fn home_dir() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/".to_string()))
}

fn antigravity_registration_json(exe_path: &str, model: &str) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&json!({
        "mcpServers": {
            "local_llm": {
                "command": exe_path,
                "args": [],
                "env": {
                    "LM_STUDIO_MODEL": model
                }
            }
        }
    }))
}

fn codex_registration_toml(exe_path: &str, model: &str) -> String {
    format!(
        "[mcp_servers.local_llm]\ncommand = {exe_path:?}\nargs = []\nstartup_timeout_sec = {startup}\ntool_timeout_sec = {tool}\n\n[mcp_servers.local_llm.env]\nLM_STUDIO_MODEL = {model:?}\n",
        startup = CODEX_STARTUP_TIMEOUT_SECS,
        tool = CODEX_TOOL_TIMEOUT_SECS,
    )
}

pub fn write_registration_artifacts(
    output_dir: &Path,
    exe_path: &str,
    model: &str,
) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let antigravity_path = output_dir.join("mcp_registration.json");
    let codex_path = output_dir.join("codex_mcp_config.toml");

    fs::write(
        &antigravity_path,
        antigravity_registration_json(exe_path, model)?,
    )?;
    fs::write(&codex_path, codex_registration_toml(exe_path, model))?;

    Ok(vec![antigravity_path, codex_path])
}

fn merge_codex_config(
    config_path: &Path,
    exe_path: &str,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = if config_path.exists() {
        let content = fs::read_to_string(config_path)?;
        content.parse::<DocumentMut>()?
    } else {
        DocumentMut::new()
    };

    let root = doc.as_table_mut();
    if !root.contains_key("mcp_servers") {
        root.insert("mcp_servers", Item::Table(Table::new()));
    }

    let mcp_servers = root["mcp_servers"]
        .as_table_mut()
        .ok_or("`mcp_servers` exists but is not a TOML table")?;
    if !mcp_servers.contains_key("local_llm") {
        mcp_servers.insert("local_llm", Item::Table(Table::new()));
    }

    let server = mcp_servers["local_llm"]
        .as_table_mut()
        .ok_or("`mcp_servers.local_llm` exists but is not a TOML table")?;
    server["command"] = value(exe_path);
    server["args"] = value(Array::new());
    server["startup_timeout_sec"] = value(CODEX_STARTUP_TIMEOUT_SECS);
    server["tool_timeout_sec"] = value(CODEX_TOOL_TIMEOUT_SECS);

    if !server.contains_key("env") {
        server.insert("env", Item::Table(Table::new()));
    }

    let env_table = server["env"]
        .as_table_mut()
        .ok_or("`mcp_servers.local_llm.env` exists but is not a TOML table")?;
    env_table["LM_STUDIO_MODEL"] = value(model);

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(config_path, doc.to_string())?;
    Ok(())
}

fn install_codex_skill(skill_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if let Some(parent) = skill_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let new_content = codex_skill_contents();
    let already_matches = skill_path.exists() && fs::read_to_string(skill_path)? == new_content;
    if already_matches {
        return Ok(false);
    }

    fs::write(skill_path, new_content)?;
    Ok(true)
}

fn merge_antigravity_config(
    mcp_config_path: &Path,
    exe_path: &str,
    model: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config: Value = if mcp_config_path.exists() {
        let data = fs::read_to_string(mcp_config_path)?;
        serde_json::from_str(&data)?
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
        if let Some(servers) = obj.get_mut("mcpServers").and_then(|v| v.as_object_mut()) {
            servers.insert("local_llm".to_string(), local_node);
        }
    }

    if let Some(parent) = mcp_config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(mcp_config_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn install_antigravity_rules(gemini_md_path: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    let mut content = if gemini_md_path.exists() {
        fs::read_to_string(gemini_md_path)?
    } else {
        String::new()
    };

    if content.contains("Privacy & Tool Routing") {
        return Ok(false);
    }

    if !content.trim().is_empty() {
        content.push_str("\n\n");
    }
    content.push_str(antigravity_rules_contents());

    if let Some(parent) = gemini_md_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(gemini_md_path, content)?;
    Ok(true)
}

fn print_generated_artifacts(artifacts: &[PathBuf]) {
    println!("\nGenerated setup files:");
    for path in artifacts {
        println!("- {}", path.display());
    }
}

fn run_manual_setup(exe_path: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    let artifacts = write_registration_artifacts(&cwd, exe_path, model)?;
    println!(
        "\nGenerated config.toml and registration snippets in {}.",
        cwd.display()
    );
    print_generated_artifacts(&artifacts);
    println!(
        "Use `codex_mcp_config.toml` for Codex and `mcp_registration.json` for Google Antigravity."
    );
    Ok(())
}

fn run_antigravity_install(exe_path: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let gemini_dir = home_dir().join(".gemini");
    let mcp_config_path = gemini_dir.join("antigravity/mcp_config.json");
    let gemini_md_path = gemini_dir.join("GEMINI.md");

    if !gemini_dir.exists() {
        println!(
            "Unable to find Antigravity configuration folder at {}. Falling back to manual mode.",
            gemini_dir.display()
        );
        return run_manual_setup(exe_path, model);
    }

    println!("\nWhat would you like to install?");
    println!("[1] MCP Config Only");
    println!("[2] Agent Rules Only");
    println!("[3] Both");
    let sub_choice = read_line("Enter your choice: ")?;

    println!(
        "\nThe following files may be modified:\n- {}\n- {}",
        mcp_config_path.display(),
        gemini_md_path.display()
    );
    let confirm = read_line("Proceed? (y/n): ")?;
    if !confirm.eq_ignore_ascii_case("y") {
        println!("Installation aborted.");
        return Ok(());
    }

    if sub_choice == "1" || sub_choice == "3" {
        merge_antigravity_config(&mcp_config_path, exe_path, model)?;
        println!("Updated MCP config at {}", mcp_config_path.display());
    }

    if sub_choice == "2" || sub_choice == "3" {
        if install_antigravity_rules(&gemini_md_path)? {
            println!("Updated GEMINI.md at {}", gemini_md_path.display());
        } else {
            println!("Local LLM rules already exist in GEMINI.md. Skipping.");
        }
    }

    Ok(())
}

fn run_codex_install(exe_path: &str, model: &str) -> Result<(), Box<dyn std::error::Error>> {
    let codex_dir = home_dir().join(".codex");
    let codex_config_path = codex_dir.join("config.toml");
    let codex_skill_path = codex_dir.join("skills/local-llm/SKILL.md");

    println!("\nWhat would you like to install?");
    println!("[1] MCP Config Only");
    println!("[2] Skill Only");
    println!("[3] Both");
    let sub_choice = read_line("Enter your choice: ")?;

    println!(
        "\nThe following files may be modified:\n- {}\n- {}",
        codex_config_path.display(),
        codex_skill_path.display()
    );
    let confirm = read_line("Proceed? (y/n): ")?;
    if !confirm.eq_ignore_ascii_case("y") {
        println!("Installation aborted.");
        return Ok(());
    }

    if sub_choice == "1" || sub_choice == "3" {
        merge_codex_config(&codex_config_path, exe_path, model)?;
        println!(
            "Updated Codex MCP config at {}",
            codex_config_path.display()
        );
    }

    if sub_choice == "2" || sub_choice == "3" {
        if install_codex_skill(&codex_skill_path)? {
            println!("Installed Codex skill at {}", codex_skill_path.display());
        } else {
            println!(
                "Codex skill is already up to date at {}.",
                codex_skill_path.display()
            );
        }
    }

    Ok(())
}

pub async fn run_interactive_installer(
    exe_path: String,
    model: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Select an option:");
    println!("[1] Setup Manually (Generate local setup files)");
    println!("[2] Auto-Install for Google Antigravity");
    println!("[3] Auto-Install for Codex");
    let choice = read_line("Enter your choice: ")?;

    match choice.as_str() {
        "1" => run_manual_setup(&exe_path, &model)?,
        "2" => run_antigravity_install(&exe_path, &model)?,
        "3" => run_codex_install(&exe_path, &model)?,
        _ => eprintln!("Invalid choice. Exiting."),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::merge_codex_config;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_temp_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("lm_bridge_{name}_{nanos}.toml"))
    }

    #[test]
    fn merge_codex_config_preserves_existing_sections() {
        let config_path = unique_temp_path("codex_merge");
        fs::write(&config_path, "[windows]\nsandbox = \"elevated\"\n").unwrap();

        merge_codex_config(
            &config_path,
            "C:\\tools\\lm-bridge.exe",
            "openai/gpt-oss-20b",
        )
        .unwrap();

        let updated = fs::read_to_string(&config_path).unwrap();
        assert!(updated.contains("[windows]"));
        assert!(updated.contains("sandbox = \"elevated\""));
        assert!(updated.contains("[mcp_servers.local_llm]"));
        assert!(updated.contains("command = "));
        assert!(updated.contains("lm-bridge.exe"));
        assert!(updated.contains("[mcp_servers.local_llm.env]"));
        assert!(updated.contains("LM_STUDIO_MODEL = \"openai/gpt-oss-20b\""));

        let _ = fs::remove_file(config_path);
    }
}
