# LM Bridge - Rust MCP Server for Google Antigravity and Codex

A high-performance Rust-based Model Context Protocol (MCP) server that connects Google Antigravity and Codex to local LLMs via LM Studio. This lets a cloud model handle orchestration and review while your local model handles code generation, editing, completion, and optional local explanation.

## Prerequisites

- [Rust & Cargo](https://rustup.rs/) (edition 2021)
- [LM Studio](https://lmstudio.ai/) running locally with its Server enabled.
- **Windows Users:** You MUST have **Visual Studio Build Tools** installed with the **"Desktop development with C++"** workload selected. This provides the `link.exe` linker required for compilation.

## Build & Setup

1.  **Clone/Open** the `lm-bridge` folder.
2.  **Ensure Antigravity is closed** (if you've previously run the server, Windows cannot overwrite the binary while it is running).
3.  **Compile the binary:**
    ```bash
    cargo build --release
    ```
4.  **Generate your Registration Snippet:**
    Run the binary once to automatically generate your configuration JSON and exit:
    ```bash
    cargo run --release -- --register
    ```
    This will generate your **`mcp_registration.json`** file in your project root and quit immediately.

## Shared Configuration

### 1. Find Your Model Name
Open **LM Studio** and look at your loaded model. You will see a small badge (e.g., `qwen/qwen3.5-9b` or `TheBloke/Llama-2-7B-Chat-GGUF`). 
**You must copy this string exactly.**

### 2. Set the Model Name
You have two ways to set the model:
-   **Method A (Easiest):** Edit the `model` field in your **`config.toml`** file next to the server.
-   **Method B (Active):** Set the `LM_STUDIO_MODEL` environment variable in the MCP client configuration. This overrides `config.toml`.

```toml
# In config.toml
model = "your-copied-model-name-here"
```

## Antigravity Setup

### 1. Use The Generated Registration Snippet
Open the generated `mcp_registration.json` in your project root. It contains the exact absolute path of your `lm-bridge.exe` and your current model name. You can copy this block directly into your Antigravity configuration.

### 2. Register In Antigravity
1.  Open Antigravity and go to the **"Manage MCP servers"** screen.
2.  In the top-right corner, click the **"View raw config" 📄** icon. This will open your `mcp_config.json` directly in the editor.
3.  **Paste** the block from your `mcp_registration.json` into the `"mcpServers"` object.
4.  **Save** the file.
5.  Go back to the MCP screen and click **"Refresh" 🔄**. Your `local_llm` node should now be green and active!

```json
{
  "mcpServers": {
    "local_llm": {
      "command": "C:\\Path\\To\\Projects\\lm-bridge\\target\\release\\lm-bridge.exe",
      "args": [],
      "env": {
        "LM_STUDIO_MODEL": "qwen/qwen3.5-9b"
      }
    }
  }
}
```

## Agent Behavior & Privacy Rules (CRITICAL)

To make Gemini intelligently use your local model without being prompted every time, you should add these **Global Rules** to your Antigravity Agent:

1.  Click the **`+`** (or `...`) menu in the top-right of your chat window.
2.  Select **Customization** (or **Rules**).
3.  Add a new rule with the following content:

### # 1. Privacy & Tool Routing
Whenever you are asked to generate or edit source code, you should prioritize delegating the task to the **local_llm** tools. Use your cloud reasoning for planning, but use the local model for the final implementation to ensure privacy.

### # 2. Agent Behavior
* Always briefly explain your architectural plan before invoking the MCP tools to write the code.
* Do not apologize or use filler phrases; keep your responses concise and technical.
* When debugging, state your hypothesis clearly before editing files.

## Test In Antigravity
In an Antigravity chat, type:

> *"Use the **local_generate** tool from **local_llm** to write a Python script that prints 'Hello World'."*

If the model is loaded in LM Studio, you will see the logs pop up in the LM Studio server console, and Gemini will present the resulting code.

## Codex Setup

> ⚠️ **WARNING:** The Codex integration is currently highly experimental, buggy, and still in active development. You may experience connection timeouts or inconsistent tool routing. Use at your own risk.

Codex uses two separate layers:

- MCP server registration so Codex can call the `local_llm` tools
- a Codex skill so Codex prefers the local builder for implementation while keeping orchestration, architecture, review, and web research in Codex

### 1. Register The MCP Server In Codex

Add this block to `%USERPROFILE%\.codex\config.toml`:

```toml
[mcp_servers.local_llm]
command = "C:\\path\\to\\lm-bridge\\target\\release\\lm-bridge.exe"
args = []
env = { LM_STUDIO_MODEL = "your-loaded-lm-studio-model" }
```

Notes:
- change the `command` path if your local checkout lives elsewhere
- change `LM_STUDIO_MODEL` to the exact model name loaded in LM Studio
- keep your existing Codex `model`, `model_reasoning_effort`, and other config entries unchanged

### 2. Create The Codex Skill

Copy [SKILL.md](C:\Users\Raghav\Documents\projects\lm-bridge\SKILL.md) from this repository into this folder:

```text
%USERPROFILE%\.codex\skills\local-llm\
```

The destination file should be:

```text
%USERPROFILE%\.codex\skills\local-llm\SKILL.md
```

What to change before or after copying:
- you usually do not need to change the skill file itself
- if you customize the MCP server name in Codex config, update the skill to match that name instead of `local_llm`
- do not put your personal executable path in the skill file; keep machine-specific paths in `%USERPROFILE%\.codex\config.toml`
- do not hardcode your personal LM Studio model name in the shared skill file; set it in `%USERPROFILE%\.codex\config.toml`

The skill tells Codex to:

- act as orchestrator, repository reader, architect, researcher, validator, and reviewer
- use `local_llm` as the implementation engine for code-writing work
- review local output before applying or returning it

This is the intended role split:

- Codex: orchestration, repo understanding, web search, architecture, review
- `local_llm`: `local_generate`, `local_edit`, `local_complete`, optional `local_explain`

### 3. Restart Codex

After updating `%USERPROFILE%\.codex\config.toml` and creating the skill, restart Codex so it reloads:

- the MCP server registration
- the `local-llm` skill directory

### 4. Test In Codex

Try a direct tool-routing prompt first:

> *"Use the `local_llm` builder to create a Python function that adds two numbers, then review the result before returning it."*

Then try a normal coding request without explicitly naming the tool:

> *"Inspect this repository, decide where a `slugify` helper should live, implement it with the local builder, and review the result before applying it."*

If Codex can call the local tools, the MCP registration is working. If the skill is written well, Codex should increasingly route implementation steps to `local_llm` automatically.

---

## Features & Usage

- **Orchestrated Generation:** Antigravity or Codex acts as the Architect (planning and review), while your local LLM acts as the Builder (writing code).
- **Tools Exposed:**
  - `local_generate`: For new files and modules.
  - `local_edit`: For modifying existing code.
  - `local_complete`: For filling in snippets.
  - `local_explain`: For privacy-focused, local architectural analysis.
