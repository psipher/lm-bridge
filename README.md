# LM Bridge

Rust MCP server for routing Codex or Google Antigravity to a local LM Studio model.

`lm-bridge` exposes local coding tools such as `local_generate` and `local_edit` over MCP. The intended split is:

- Codex or Antigravity handles orchestration, planning, and review.
- Your local LM Studio model handles the code-writing step.

The project is now documented for Codex first. Antigravity remains supported.

## What Codex expects

OpenAI's current Codex MCP docs say Codex MCP servers are configured in `~/.codex/config.toml` or project `.codex/config.toml`, under `[mcp_servers.<name>]`, with `command`, optional `args`, optional `env`, optional `cwd`, and optional timeout fields. Reference: [Model Context Protocol – Codex](https://developers.openai.com/codex/mcp).

`lm-bridge` now aligns with that model:

- the installer can update `%USERPROFILE%\.codex\config.toml`
- the project can generate a Codex-ready TOML snippet in `codex_mcp_config.toml`
- the repo includes a Codex skill at `integrations/codex/SKILL.md`
- the repo also includes an evaluation-oriented Codex skill at `integrations/codex-eval/SKILL.md`

## Prerequisites

- [LM Studio](https://lmstudio.ai/) running locally with its server enabled
- a model loaded in LM Studio
- Rust only if you want to build from source

## Recommended LM Studio settings

The default `config.toml` in this repo is tuned for `openai/gpt-oss-20b`.

Suggested baseline on an RTX 5070 Ti / 64 GB RAM setup:

- Context Length: `32768`
- GPU Offload: `24`
- Unified KV Cache: `ON`
- Offload KV Cache to GPU Memory: `ON`
- Number of Experts: `4`
- Evaluation Batch Size: `512`
- Temperature: `0.1` or `0.2`
- Top P: `0.8`
- Min P: `0.05`

Do not use reasoning models that emit `<think>` or long hidden reasoning blocks. They are a bad fit for tool JSON output and will often break downstream parsing.

## Build

### Prebuilt binary

1. Download the latest release for your platform.
2. Put the binary in its own folder.
3. Run it once. If `config.toml` does not exist next to the executable, it will be created automatically.
4. Run the binary interactively to choose manual setup, Antigravity install, or Codex install.

### Build from source

Requirements:

- Rust and Cargo
- Windows: Visual Studio Build Tools with Desktop development with C++
- macOS: Xcode Command Line Tools

Build:

```bash
cargo build --release
```

Generate setup snippets without starting the MCP server:

```bash
cargo run --release -- --register
```

That creates these files next to `config.toml`:

- `codex_mcp_config.toml`
- `mcp_registration.json`

Run a fast local diagnostic without starting the MCP loop:

```bash
cargo run --release -- --self-test
```

That self-test reports:

- resolved `config.toml` path
- resolved executable path
- current LM Studio URL and configured model
- whether Codex config and skill files exist
- whether `LM Studio /v1/models` is reachable
- whether the configured model is currently exposed by LM Studio

## Shared model configuration

Open LM Studio, copy the exact loaded model identifier, and set it in one of these places:

- `config.toml` next to `lm-bridge`
- `LM_STUDIO_MODEL` in your MCP client config

Example:

```toml
model = "openai/gpt-oss-20b"
```

Client-side `LM_STUDIO_MODEL` overrides the local `config.toml` value.

## Codex setup

### Option 1: Auto-install from the interactive installer

Run the executable directly and choose:

1. `[3] Auto-Install for Codex`
2. `[1] MCP Config Only`, `[2] Skill Only`, or `[3] Both`

The installer writes:

- `%USERPROFILE%\.codex\config.toml`
- `%USERPROFILE%\.codex\skills\local-llm\SKILL.md`

The MCP server is registered as `local_llm`.

### Option 2: Manual Codex setup

Copy the generated `codex_mcp_config.toml` snippet into your `~/.codex/config.toml`.

Example:

```toml
[mcp_servers.local_llm]
command = "C:\\Path\\To\\lm-bridge.exe"
args = []
startup_timeout_sec = 20
tool_timeout_sec = 120

[mcp_servers.local_llm.env]
LM_STUDIO_MODEL = "openai/gpt-oss-20b"
```

Then install the included skill by copying:

- repo source: `integrations/codex/SKILL.md`
- destination: `%USERPROFILE%\.codex\skills\local-llm\SKILL.md`

### Optional evaluation skill

If you want the local model to stay the primary code owner for longer before Codex falls back, install the evaluation skill instead:

- repo source: `integrations/codex-eval/SKILL.md`
- suggested destination: `%USERPROFILE%\.codex\skills\local-llm-eval\SKILL.md`

Use the default `local-llm` skill for normal production work. Use `local-llm-eval` when you want stricter benchmarking behavior with at least two defect-specific repair attempts before Codex takes over implementation.

### Verify in Codex

1. Restart Codex after changing MCP config.
2. Open `/mcp` and confirm `local_llm` is active.
3. Ask Codex to use `local_generate` or perform a coding task that should route to the local builder skill.

If you want timing diagnostics for startup or tool calls, start the server with `LM_BRIDGE_DEBUG=1`.

Before debugging Codex itself, run:

```bash
cargo run --release -- --self-test
```

If `configured_model_present: no`, the problem is in LM Studio or the configured model name, not in Codex MCP registration.

## Google Antigravity setup

### Option 1: Auto-install from the interactive installer

Run the executable and choose:

1. `[2] Auto-Install for Google Antigravity`
2. `[1] MCP Config Only`, `[2] Agent Rules Only`, or `[3] Both`

The installer updates:

- `~/.gemini/antigravity/mcp_config.json`
- `~/.gemini/GEMINI.md`

### Option 2: Manual setup

Open the generated `mcp_registration.json` and paste the `local_llm` block into Antigravity's `mcp_config.json`.

Example:

```json
{
  "mcpServers": {
    "local_llm": {
      "command": "C:\\Path\\To\\lm-bridge.exe",
      "args": [],
      "env": {
        "LM_STUDIO_MODEL": "openai/gpt-oss-20b"
      }
    }
  }
}
```

For agent behavior, use the rules in `integrations/google_antigravity/GEMINI.md`.

## Tools exposed

- `local_generate`
- `local_edit`
- `local_complete`
- `local_explain`

## Startup and latency notes

To reduce Codex-side latency, normal MCP startup no longer generates registration files and no longer performs background update checks. Registration artifacts are only generated in manual setup or `--register` flows.

If Codex still feels slow to invoke the server, check these first:

- LM Studio is already running
- the configured model is loaded
- the `LM_STUDIO_MODEL` value matches the exact LM Studio model name
- Codex is pointing at the release binary, not a slow debug build

## Current focus

- improve Codex integration quality
- keep Antigravity support working
- add support for more local model presets
