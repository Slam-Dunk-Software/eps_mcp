# eps_mcp — Customization Guide

`eps_mcp` is an MCP (Model Context Protocol) server that gives Claude authoritative knowledge about the EPS (Extremely Personal Software) ecosystem. Install it once and Claude gains access to ADRs, design concepts, package examples, and harness templates without needing web access.

## Installing

```bash
epm mcp install eps_mcp
```

This builds the binary and registers it in `~/.claude.json`. Restart Claude to activate.

## Ports (Extension Points)

### Knowledge Sources

**PORT: EPS Registry URL**
The server fetches live package data from the EPS registry. To point it at a private or local registry, set `EPS_REGISTRY_URL` in the env section of your `~/.claude.json` entry:

```json
"eps_mcp": {
  "command": "/path/to/eps_mcp",
  "args": [],
  "env": {
    "EPS_REGISTRY_URL": "http://localhost:3001"
  }
}
```

Default: `https://epm.dev`

### Tools Provided

The server exposes these tools to Claude:

- `get_overview()` — high-level EPS ecosystem summary
- `get_adr(number)` — read a specific Architecture Decision Record
- `list_adrs()` — browse all ADRs
- `get_concept(name)` — deep-dive on a core EPS concept
- `get_example(name)` — canonical harness examples
- `get_eps_toml_reference()` — full eps.toml field reference
- `get_customize_md_template()` — CUSTOMIZE.md template for new harnesses
- `list_epc_docs()` — list available EPC runtime docs
- `get_epc_doc(path)` — read a specific EPC doc
- `list_persona_components()` — browse persona component registry
- `search_persona_components(query)` — search components
- `get_persona_component(name)` — component details + source
- `install_persona_component(name, persona_path)` — download to components/
- `publish_persona_component(...)` — upload to registry

**PORT: New Tools**
Add your own tools by implementing the `rmcp` tool trait in `src/tools/`. Each tool is a separate module. See existing tools for the pattern.

### Persona Registry

**PORT: PERSONA_REGISTRY_URL**
The persona component tools default to `http://100.78.103.79:3004`. Override with:

```json
"env": { "PERSONA_REGISTRY_URL": "http://your-registry:3004" }
```

## Uninstalling

```bash
epm mcp remove eps_mcp
```

Then restart Claude.
