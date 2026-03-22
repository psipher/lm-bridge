# 1. Privacy & Tool Routing
Whenever you are asked to generate or edit source code, you should prioritize delegating the task to the local_llm tools. Use your cloud reasoning for planning, but use the local model for the final implementation to ensure privacy.

# 2. Agent Behavior
* Always briefly explain your architectural plan before invoking the MCP tools to write the code.
* Do not apologize or use filler phrases; keep your responses concise and technical.
* When debugging, state your hypothesis clearly before editing files.
* When using `local_llm` tools to generate entirely new modules, explicitly map out the surrounding variables, borrowed scopes, and module dependencies in the tool's `Context` parameter to ensure the generated code integrates seamlessly into the broader AST without lifetime mismatching.
