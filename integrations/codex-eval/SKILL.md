---
name: local-llm-eval
description: |
  Use this skill when the goal is to evaluate local_llm as the primary coding model rather than just use it as a fast implementation helper. Trigger when the user wants Codex to minimize manual rewriting, benchmark local generation quality, or keep ownership of implementation with the local model for multiple repair rounds before fallback.
---

# Operating Model

Use this role split when `local_llm` is being evaluated as the main code writer:
- Codex handles orchestration, repository analysis, review, and defect reporting.
- `local_llm` owns the implementation attempts and repair iterations.

Treat Codex as the reviewer and escalation path.
Treat `local_llm` as the default code owner for the task.

# Evaluation Workflow

1. Understand the request and gather only the minimum context needed to choose the right local tool.
2. Do brief planning in Codex, but minimize Codex-authored implementation.
3. For coding work, call the matching `local_llm` MCP tool promptly.
4. Review the returned code for correctness, project fit, and regressions.
5. If review finds defects, send a defect-specific correction prompt back to `local_llm` that enumerates the exact issues.
6. Repeat repair through `local_llm` before falling back to Codex-authored implementation.
7. In the final response, distinguish accepted local output from any Codex fallback work.

# Routing Rules

Prefer `local_llm` for:
- implementing new code
- editing existing code
- refactoring code
- fixing bugs through code changes
- completing partial implementations
- generating boilerplate or larger code blocks

Keep work in Codex for:
- deciding what to build
- comparing design options
- reading the repository and identifying constraints
- reviewing generated code for defects or mismatches
- deciding whether the local output is acceptable

# Tool Selection

Use `local_generate` for new files, new modules, and substantial new implementations.

Use `local_edit` by default for changes to existing files. Only use `local_generate` instead when the existing file is being replaced wholesale.

Use `local_complete` for continuing unfinished snippets.

Use `local_explain` only when a local explanation is specifically useful or requested.

# Conformance Requirements

When calling `local_llm`, explicitly require:
- syntactically valid code in the target language
- compatibility with the existing repository structure
- adherence to stated repository conventions
- no placeholder or pseudo-code
- exact correction of listed review defects when retrying

# Review Rules

Never present local tool output blindly.

Review each local implementation for:
- syntax validity
- instruction compliance
- project fit
- missing imports or dependencies
- runtime or build plausibility when relevant
- logic correctness
- regressions relative to the request

# Fallback Behavior

Minimize Codex-authored code changes during evaluation mode.

Do at least two defect-specific repair attempts with `local_llm` before Codex manually rewrites the implementation, unless:
- the tool is unavailable
- the output is fundamentally unusable
- the task is blocked by missing context or an external dependency

If fallback is required, keep the fallback narrow and explicitly note that Codex had to take over implementation.

# Configuration Notes

Expected MCP server name: `local_llm`

Do not put a machine-specific executable path in this skill file.

Do not put a machine-specific LM Studio model name in this skill file.

Install this skill separately from the default production-oriented `local-llm` skill when you want evaluation behavior.
