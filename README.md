# LM Bridge - Rust MCP Server

A high-performance Rust-based Model Context Protocol (MCP) server that connects Google Antigravity to local LLMs via LM Studio. This allows Antigravity (powered by Gemini) to offload heavy code generation, editing, and explanation tasks to your local hardware.

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

## Configuration

### 1. `mcp_registration.json` (The Shortcut)
Open the generated `mcp_registration.json` in your project root. It contains the **exact absolute path** of your `lm-bridge.exe` and your current model name. You can copy this block directly into your Antigravity configuration.

### 1. Find Your Model Name
Open **LM Studio** and look at your loaded model. You will see a small badge (e.g., `qwen/qwen3.5-9b` or `TheBloke/Llama-2-7B-Chat-GGUF`). 
**You must copy this string exactly.**

### 2. Set the Model Name
You have two ways to set the model:
-   **Method A (Easiest):** Edit the `model` field in your **`config.toml`** file.
-   **Method B (Active):** Edit the `LM_STUDIO_MODEL` environment variable in your **`mcp_config.json`**. (This will always override your `config.toml`).

```toml
# In config.toml
model = "your-copied-model-name-here"
```

### 3. Antigravity Registration (The Final Step)
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

---

## Features & Usage

- **Orchestrated Generation:** Gemini acts as the Architect (planning), while your local LLM acts as the Builder (writing code).
- **Tools Exposed:**
  - `local_generate`: For new files and modules.
  - `local_edit`: For modifying existing code.
  - `local_complete`: For filling in snippets.
  - `local_explain`: For privacy-focused, local architectural analysis.

### Testing the Connection
In an Antigravity chat, type:
> *"Use the **local_generate** tool from **local_llm** to write a Python script that prints 'Hello World'."*

If the model is loaded in LM Studio, you will see the logs pop up in the LM Studio server console, and Gemini will present the resulting code!
