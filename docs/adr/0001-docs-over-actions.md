# ADR-0001: Docs-Over-Actions Architecture

- **Date:** 2026-02-28
- **Status:** Accepted

## Context

When designing eps-mcp, two architectural models were on the table:

**Model A — Action MCP:** Expose tools that execute EPM and EPC commands on the user's
behalf (`run_epm(args)`, `run_epc(args)`, `validate_eps_toml(path)`, etc.). The LLM
becomes an orchestrator that reads state and issues commands.

**Model B — Docs MCP:** Expose tools that return authoritative knowledge about the EPS
ecosystem — ADRs, concept explanations, file format references, and canonical examples.
The LLM reasons from accurate knowledge rather than from training data.

The question was raised directly during development: *is action execution actually what
makes this useful, or is the knowledge layer the real value?*

The comparison that settled it: the Claude developer docs MCP is a pure knowledge MCP —
no command execution, no filesystem access — and it's highly effective. Its value comes
entirely from giving the LLM accurate, current documentation that training data doesn't
reliably contain. EPS is in exactly the same position: it's brand new, the LLM has no
training data on it, and every design decision lives in the ADRs.

A secondary observation: Claude Code already has Bash tool access. An LLM helping a user
build an EPS can already run `epm init`, `epm publish`, and `epc deploy` directly. The
bottleneck is never "can I run the command" — it's "do I know what the right command is,
what the right file structure looks like, and whether the user's files are correct."

## Decision

eps-mcp is a **docs MCP**. All tools return text. No subprocess execution, no filesystem
reads, no system state.

The gap identified in the initial implementation was not missing action tools — it was
missing practical reference material. The ADR corpus covers *why* decisions were made,
but an LLM helping a user build an EPS also needs:

- The complete `eps.toml` schema (not just the ADR that introduced it)
- A fill-in-the-blanks `CUSTOMIZE.md` template
- Canonical examples showing what a real `eps.toml` + `CUSTOMIZE.md` looks like

These were added as new tools: `get_eps_toml_reference`, `get_customize_md_template`,
and `get_example`. They follow the same pattern as the existing knowledge tools.

## Consequences

**Positive:**
- The server is trivially simple — no subprocess management, no filesystem access,
  no security surface, no platform-specific behavior
- Content improvements (new tools, updated docs) require only a Rust recompile, not
  architectural changes
- Works identically on any machine, regardless of whether EPM or EPC is installed
- The docs stay accurate because they're maintained alongside the ecosystem ADRs

**Negative:**
- The LLM must still use Bash tool to actually run EPM/EPC commands; eps-mcp can't
  close that loop itself
- Content must be kept current as EPM/EPC CLIs evolve; stale docs are worse than no docs
- The LLM cannot introspect the user's actual project state (e.g., read their eps.toml
  to catch errors); it can only provide reference material and let the user compare

**If this proves insufficient:** Action tools can be layered on top without replacing the
knowledge layer. The two models are not mutually exclusive — but the knowledge layer
should be built first and built well before adding the complexity of command execution.
