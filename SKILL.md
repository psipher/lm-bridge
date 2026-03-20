---
name: local-llm
description: |
  Use this skill when Codex should act as orchestrator, researcher, architect, and reviewer, while delegating code-writing work to the local_llm MCP server. Trigger for coding tasks such as writing code, implementing features, editing files, refactoring, fixing bugs, completing snippets, and explaining code locally, especially when the goal is to keep implementation on a local model while retaining Codex for planning and validation.
---

# Operating Model

Use this role split by default when `local_llm` is available:
- Codex handles orchestration, repository analysis, architecture, web research when needed, validation, and review.
- `local_llm` handles code generation, code transformation, snippet completion, and optional local code explanation.

Treat Codex as the decision-maker and reviewer.
Treat `local_llm` as the implementation engine.

# Default Workflow

1. Understand the request and gather only the minimum context needed to choose the right local tool.
2. Do only brief planning in Codex unless the task is genuinely ambiguous or high risk.
3. For straightforward coding work, call the matching `local_llm` MCP tool promptly instead of delaying on extended reasoning.
4. Review the returned code in Codex for correctness, project fit, and regressions.
5. If needed, refine the prompt and call the local tool again.
6. Present or apply only the reviewed result.

# Routing Rules

Prefer `local_llm` for these tasks:
- implementing new code
- editing existing code
- refactoring code
- fixing bugs through code changes
- generating boilerplate or larger code blocks
- completing partial implementations
- explaining code locally when requested

Keep work in Codex for these tasks:
- deciding what to build
- comparing design options
- reading the repository and identifying constraints
- reviewing generated code for defects or mismatches
- searching documentation or the web when current information matters
- deciding whether the generated code should be accepted or revised

Prefer early delegation to `local_llm` when the task is already well specified and does not need deep architectural analysis before implementation.

Only skip `local_llm` when:
- the MCP server is unavailable
- the task is not actually a coding task
- the user explicitly asks not to use the local model
- the implementation is so small or incidental that delegation adds no value

# Tool Selection

Use `local_generate` for new files, new functions, modules, feature implementations, and substantial code generation from a task description.

Do not use `local_generate` as a fallback for edits to an existing file when the actual task is code modification inside that file.

Use `local_edit` for modifying existing code when you can provide the current code and a precise instruction.

Prefer `local_edit` over `local_generate` for in-place changes to an existing file, component, module, handler, or function.

Use `local_complete` for filling in partial code or continuing an unfinished snippet.

Use `local_explain` only when a local explanation is specifically useful or requested.

# Prompting Rules For Local Tools

When calling `local_llm`, provide implementation-ready inputs:
- the exact coding objective
- language and file path when relevant
- surrounding code or constraints
- architectural decisions already made by Codex
- any repository conventions the generated code must follow

Do not offload vague planning to the local model when Codex can resolve it first, but do not spend excessive time planning before a straightforward implementation call.

# Review Rules

Never present local tool output blindly.

Codex must review generated code for:
- correctness
- missing imports or dependencies
- API misuse
- mismatch with repository patterns
- incomplete edge-case handling
- regressions relative to the request

If the result is weak, retry with a tighter prompt before falling back.

# Fallback Behavior

If `local_llm` is unavailable, continue with the normal Codex workflow and mention briefly that the local builder path was not active.

If the local tool fails repeatedly, keep orchestration and review in Codex and proceed with the best available fallback.

If `local_edit` returns an empty or unusable result, treat that as a failed edit attempt rather than a valid answer.

# Configuration Notes

Expected MCP server name: `local_llm`

Do not put a machine-specific executable path in this skill file.

Do not put a machine-specific LM Studio model name in this skill file.

The `local_llm` MCP server must already be registered in Codex and point to the user's local `lm-bridge` executable.

Set the executable path and `LM_STUDIO_MODEL` in the user's Codex MCP configuration, usually `%USERPROFILE%\.codex\config.toml`.

Only change this skill if the MCP server is registered under a different name than `local_llm`.
