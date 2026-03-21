# LM Bridge - Rust MCP Server for Google Antigravity

A high-performance Rust-based Model Context Protocol (MCP) server that connects Google Antigravity to local LLMs via LM Studio. This lets a cloud model handle orchestration and review while your local model handles code generation, editing, completion, and optional local explanation.

*Optimized for `openai/gpt-oss-20b` and  tested on an RTX 5070 Ti (64GB RAM).*

## Prerequisites

- [LM Studio](https://lmstudio.ai/) running locally with its Server enabled.

## Hardware & LM Studio Configuration (RTX 5070 Ti / 64GB RAM)

To achieve maximum performance with `gpt-oss-20b`, use the following hardware and inference settings in LM Studio:

### Context and Offload
- **Context Length:** `32768`
- **GPU Offload:** `24` (Ensures the entire model sits in the 5070 Ti's high-bandwidth GDDR7 memory)
- **Unified KV Cache:** `ON` (Allows system RAM to act as a spillover for large context windows)
- **Offload KV Cache to GPU Memory:** `ON` (Prioritizes keeping the active context on the GPU for faster inference)
- **Number of Experts:** `4` (Maintains optimal speed-to-intelligence ratio)
- **Evaluation Batch Size:** `512` (Optimal balance for Tensor Cores)

### Inference & Sampling Settings
Set these in the right-hand panel of LM Studio to force the model into a strict, deterministic "Worker" mode:
- **Temperature:** `0.1` or `0.2` (Lowers creativity to prevent syntax errors or hallucinations in code logic)
- **Top P Sampling:** `0.8` (Balances precision without getting stuck in loops)
- **Min P Sampling:** `0.05` (Prunes low-probability noise for higher quality snippets)
- **Reasoning Section Parsing:** `ON` (Allows you to see the worker's internal logic before generating the code block)

### Harmony Chat Format (System Prompt)
GPT-OSS is trained on the Harmony Chat Format. By default, `lm-bridge`'s included `config.toml` injects the necessary `"Worker"` persona into every code generation prompt automatically. **You do not need to configure a custom System Prompt in LM Studio.** The bridge handles the architecture delegation natively.

## Installation & Setup

### Option 1: Download Pre-Built Binary (Recommended)
1. Go to the [Releases screen](https://github.com/) on GitHub and download the `.exe` (or macOS/Linux binary) for your operating system.
2. Place the binary inside a new folder (e.g., `lm-bridge`).
3. Download the example `config.toml` from the repository and place it next to the binary.

### Option 2: Build From Source

**Build Requirements:**
- [Rust & Cargo](https://rustup.rs/) (edition 2021)
- **Windows Users:** You MUST have **Visual Studio Build Tools** installed with the **"Desktop development with C++"** workload selected. This provides the `link.exe` linker required for compilation.
- **macOS Users:** You MUST have Xcode Command Line Tools installed. You can install them by running `xcode-select --install` in your terminal.

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
Open **LM Studio** and look at your loaded model. You will see a small badge (e.g., `openai/gpt-oss-20b`. 
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
      "command": "C:\\Path\\To\\Projects\\lm-bridge\\target\\release\\lm-bridge.exe", // macOS: "/Users/Name/Projects/lm-bridge/target/release/lm-bridge"
      "args": [],
      "env": {
        "LM_STUDIO_MODEL": "openai/gpt-oss-20b"
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
The MCP server works without mentioning the tool name in the prompt.
## Features & Usage

- **Orchestrated Generation:** Antigravity acts as the Architect (planning and review), while your local LLM acts as the Builder (writing code).
- **Tools Exposed:**
  - `local_generate`: For new files and modules.
  - `local_edit`: For modifying existing code.
  - `local_complete`: For filling in snippets.
  - `local_explain`: For privacy-focused, local architectural analysis.

## Currently in development
* Make it work with codex app
* Make it work with multiple local models

