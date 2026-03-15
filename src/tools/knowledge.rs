// Static knowledge corpus for the EPS ecosystem.
//
// Concept docs and ADRs are fetched at runtime from the public
// epm_registry GitHub repo (docs/ directory), so eps-mcp is self-contained
// and docs stay current without a rebuild.
//
// EPC-specific documentation lives in the epc repo and is served from the
// local filesystem via list_epc_docs() / get_epc_doc().

use std::path::{Component, Path, PathBuf};

// ── EPM docs: fetched at runtime from public GitHub repo ──────────────────────

const EPM_DOCS_BASE: &str =
    "https://raw.githubusercontent.com/nickagliano/epm_registry/main";

fn fetch_from_eps_docs(path: &str) -> String {
    let url = format!("{EPM_DOCS_BASE}/{path}");
    match ureq::get(&url).call() {
        Ok(resp) => resp.into_string().unwrap_or_else(|_| {
            format!("(failed to decode response from {url})")
        }),
        Err(e) => format!(
            "Could not fetch EPS docs from GitHub.\nURL: {url}\nError: {e}\n\n\
             Make sure you have internet access and that the epm_registry \
             repo is public at github.com/nickagliano/epm_registry."
        ),
    }
}

// ── EPC live-docs helpers ──────────────────────────────────────────────────────

fn epc_docs_path() -> PathBuf {
    if let Ok(path) = std::env::var("EPC_DOCS_PATH") {
        PathBuf::from(path)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join("Documents/personal-projects/epc/docs")
    } else {
        PathBuf::from("/tmp/epc-docs-not-found")
    }
}

/// Strip any `..`, leading `/`, or `.` components so callers can't escape the docs dir.
fn sanitize_doc_path(raw: &str) -> PathBuf {
    let mut out = PathBuf::new();
    for component in Path::new(raw).components() {
        if let Component::Normal(c) = component {
            out.push(c);
        }
    }
    out
}

fn collect_docs(base: &Path, dir: &Path, paths: &mut Vec<String>) {
    let mut entries: Vec<_> = match std::fs::read_dir(dir) {
        Ok(e) => e.filter_map(|e| e.ok()).collect(),
        Err(_) => return,
    };
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_docs(base, &path, paths);
        } else if path.extension().map(|e| e == "md").unwrap_or(false) {
            if let Ok(rel) = path.strip_prefix(base) {
                paths.push(rel.to_string_lossy().to_string());
            }
        }
    }
}

pub fn list_epc_docs() -> String {
    let base = epc_docs_path();
    if !base.exists() {
        return format!(
            "EPC docs directory not found at {}.\n\
             Set the EPC_DOCS_PATH environment variable to the path of the epc/docs/ directory.",
            base.display()
        );
    }
    let mut paths = Vec::new();
    collect_docs(&base, &base, &mut paths);
    let mut out = format!(
        "# EPC Documentation Index\n\
         source: {}\n\n\
         Pass any path below to `get_epc_doc`:\n\n",
        base.display()
    );
    for p in &paths {
        out.push_str(&format!("  {p}\n"));
    }
    out
}

pub fn get_epc_doc(doc_path: &str) -> String {
    let base = epc_docs_path();
    let safe_rel = sanitize_doc_path(doc_path);
    let full_path = base.join(&safe_rel);
    match std::fs::read_to_string(&full_path) {
        Ok(content) => content,
        Err(_) => format!(
            "Doc not found: '{}'. Call list_epc_docs() to see available paths.",
            doc_path
        ),
    }
}

// ── Top-level overview ────────────────────────────────────────────────────────

pub const OVERVIEW: &str = r#"
# Extremely Personal Software (EPS) — Ecosystem Overview

## What is EPS?

Extremely Personal Software is a category of software designed to be **functional by
default but deliberately incomplete**. An EPS is a *harness*: it works out of the box,
but the default state is a starting point, not a destination. The author consciously
withheld features — not because they couldn't build them, but because the extension
surface is where the value lives.

Think of it as a motherboard, not a device. A motherboard powers on without RAM. The
BIOS runs. It does things. But nobody ships it as a finished computer — the ports are
the product. An EPS author made the same deliberate choice: they left room, and that
room is what you customize.

## The Three-Layer Stack

```
EPS   — the app/harness format (what you install and customize)
EPM   — the package manager (install, search, publish)
EPC   — the runtime (deploy, run, manage as persistent services)
```

### Layer 1 — EPS (Extremely Personal Software)

The *format*. An EPS is a git repository containing at minimum:

- `eps.toml` — machine-readable manifest (name, version, platform, hooks, etc.)
- `CUSTOMIZE.md` — required human+LLM-readable documentation of every extension point
- Source code that is functional but deliberately incomplete

An EPS has three properties:
1. **Intentional ports** — extension points are named, documented, and designed in
2. **Minimal defaults** — works out of the box, but the defaults are a starting point
3. **Interface is the value** — worth comes from what it enables, not what it does

### Layer 2 — EPM (Extremely Personal Manager)

The *package manager*. EPM is to EPS what Homebrew is to macOS packages or Cargo is to
Rust crates. The CLI (`epm`) and the centralized registry work together:

- `epm search <query>` — search the registry
- `epm info <pkg>` — show manifest, CUSTOMIZE.md, and system deps before installing
- `epm install <pkg>[@version]` — fetch, build, and install an EPS
- `epm init <name>` — scaffold a new EPS (eps.toml + CUSTOMIZE.md + run.sh)
- `epm publish` — validate and publish to the registry

The registry holds metadata and git source URLs (like crates.io). Package names are
flat and first-come-first-served (e.g. `tech_talker`, not `nickagliano/tech_talker`).
Authentication is via GitHub OAuth. EPM enforces quality gates at publish time:
`CUSTOMIZE.md` must exist, platform targets must be declared, and licensing must be
from the approved list.

### Layer 3 — EPC (Extremely Personal Cloud)

The *runtime*. EPC deploys EPS packages as persistent daemons on your own hardware,
networked via Tailscale. It is itself an EPS — dogfooding the format. Key commands:

- `epc deploy <spec>` — install an EPS and start it as a daemon
- `epc ps` — list running services with their Tailscale URLs
- `epc logs <name>` — tail stdout/stderr
- `epc stop <name>` — stop a running service

EPC turns the EPS ecosystem into a personal PaaS: all the UX of Heroku or Render,
running entirely on your own hardware with no monthly bill.

## Why EPS Exists

The software moat is gone. LLMs and agentic AI mean that feature-richness no longer
provides a lasting competitive advantage — AI can help replicate or work around any
feature set. But *customizability* is different. An EPS is valuable precisely because
it hands control back to the user: the skeleton is theirs to fill, and LLMs can help
fill it.

EPSs are also LLM-friendly by design. Every published EPS must include `CUSTOMIZE.md`,
a structured guide that an AI agent can read to understand how to configure, extend, and
modify the harness. A user can ask "how do I make tech_talker use a different hotkey?"
and the agent can answer accurately by reading the EPS rather than guessing from
training data.

## EPS Seasonings

Portable, LLM-ready implementation patterns for EPS apps. Each seasoning is a markdown file
describing a single capability. Fetch one from https://github.com/nickagliano/eps_seasonings
and tell Claude to apply it to the target project.

## What EPS Is Not

EPS is **not** "npm for agents."

There is a growing category of tooling aimed at sharing skills between autonomous agents —
packages that give agents new capabilities, compose into pipelines, or unlock new behaviors
for AI software. That is a real and interesting problem. EPS is not solving it.

EPS solves software personalization for real humans. The human is always the end
beneficiary. An agent (LLM, AI assistant, Claude) might help you install, configure,
or extend an EPS — but the agent is the tool, and you are the point.

The unit of value in EPS is not "what can an agent do with this?" It is "what does
this do for the person who installed it?"

This distinction determines what gets built and why. A package that makes agents more
capable but improves no human's daily life is agent infrastructure — not an EPS.
The question every EPS author should ask is: *whose life does this improve, and how?*

## Canonical EPS Examples

**tech_talker** — A macOS Swift audio transcription harness. Ships with Whisper (local)
and FluidAudio (streaming) support, global hotkeys, and a model manager. Ports: voice
command mappings, transcription engine, Whisper model, global shortcut key.

**pi** (powers OpenClaw, an open-source Claude Code alternative) — A minimal agent
harness. Ships with 4 tools: read, write, edit, bash. Ports: tool registration, slash
commands, TUI components, system prompt. The entire point is the extension surface.

**todo** — A local task + reading list manager with a web UI. Ships with JSON storage
and plain text output, served over HTTP. Ports: storage backend (Storage trait), output
formatter (Formatter trait), lifecycle hooks, reading list.
"#;

// ── ADR index ─────────────────────────────────────────────────────────────────

pub const ADR_INDEX: &str = r#"
# EPS Architecture Decision Records (ADRs)

ADRs capture the significant design decisions made in building the EPS ecosystem.
Use `get_adr(number)` to read the full text of any ADR.

| # | Title | Status |
|---|-------|--------|
| 1 | Use Rust as the Primary Implementation Language | Accepted |
| 2 | Centralized Registry Model | Accepted |
| 3 | EPS Package Manifest Format (`eps.toml`) | Accepted |
| 4 | CLI Tool Named `epm` | Accepted |
| 5 | LLM-Friendliness as a First-Class EPS Requirement | Accepted |
| 6 | EPS Acceptance Standards and Platform Compatibility | Draft |
| 7 | Licensing Philosophy | Draft |
| 8 | The Harness Definition — What Makes an EPS an EPS | Draft |
| 9 | Install Lifecycle | Draft |
| 10 | Registry Auth and Package Namespacing | Draft |
| 11 | Supply Chain Security | Draft |
| 12 | mdBook as Recommended EPS Documentation Format | Draft |
| 13 | System Dependency Declaration | Accepted |
| 14 | `epm init` — Package Scaffolding Command | Accepted |

**Key ADRs to read first:**
- ADR-8: The canonical definition of what an EPS is (the harness definition)
- ADR-5: Why CUSTOMIZE.md is required and what it must contain
- ADR-3: The eps.toml manifest format
- ADR-9: How `epm install` works end-to-end
"#;

// ── ADR full text (embedded at compile time) ──────────────────────────────────

pub fn get_adr(number: u32) -> String {
    let filename = match number {
        1  => "0001-rust-primary-language.md",
        2  => "0002-centralized-registry.md",
        3  => "0003-eps-manifest-format.md",
        4  => "0004-cli-named-epm.md",
        5  => "0005-llm-friendliness.md",
        6  => "0006-eps-acceptance-standards.md",
        7  => "0007-licensing-philosophy.md",
        8  => "0008-harness-definition.md",
        9  => "0009-install-lifecycle.md",
        10 => "0010-registry-auth-and-namespacing.md",
        11 => "0011-supply-chain-security.md",
        12 => "0012-mdbook-documentation.md",
        13 => "0013-system-dependency-declaration.md",
        14 => "0014-epm-init-scaffolding.md",
        _  => return format!(
            "ADR-{number:04} not found. Valid range: 1–14. Call `list_adrs` to see the full index."
        ),
    };
    fetch_from_eps_docs(&format!("docs/adr/{filename}"))
}

// ── Concept explanations ──────────────────────────────────────────────────────

fn eps_concept() -> String {
    fetch_from_eps_docs("docs/concepts/what-is-eps.md")
}

fn port_concept() -> String {
    fetch_from_eps_docs("docs/concepts/ports.md")
}

fn customize_md_concept() -> String {
    fetch_from_eps_docs("docs/concepts/customize-md.md")
}

pub fn get_concept(concept: &str) -> String {
    let key = concept.to_lowercase().replace('-', "_");
    match key.as_str() {
        "eps" => eps_concept(),
        "epm" => EPM_CONCEPT.to_string(),
        "epc" => EPC_CONCEPT.to_string(),
        "port" | "ports" => port_concept(),
        "harness" => eps_concept(),
        "customize_md" | "customize.md" => customize_md_concept(),
        "litmus_test" | "litmus" => eps_concept(),
        "install_lifecycle" | "install" => INSTALL_LIFECYCLE_CONCEPT.to_string(),
        "observatory" => OBSERVATORY_CONCEPT.to_string(),
        "seasonings" | "seasoning" => SEASONINGS_CONCEPT.to_string(),
        _ => format!(
            "Unknown concept '{concept}'. Valid options: eps, epm, epc, port, harness, \
             customize_md, litmus_test, install_lifecycle, observatory, seasonings."
        ),
    }
}

const EPM_CONCEPT: &str = r#"
# Concept: EPM (Extremely Personal Manager)

EPM is the package manager for the EPS ecosystem. It is analogous to Homebrew for macOS
packages or Cargo for Rust crates — it handles discovery, installation, and publishing.

## Components

**Registry** — A centralized index (like crates.io) that stores package metadata and
points to git source URLs. The registry does not host code; it indexes it. Built in Ruby
on Rails with PostgreSQL.

**CLI** (`epm`) — The command-line tool users interact with. Key commands:

| Command | What it does |
|---------|-------------|
| `epm search <query>` | Search the registry for EPSs |
| `epm info <pkg>` | Show manifest, CUSTOMIZE.md, system deps |
| `epm install <pkg>[@ver]` | Fetch, build, and install an EPS |
| `epm init <name>` | Scaffold a new EPS project |
| `epm publish` | Validate and publish to the registry |
| `epm list` | Show locally installed EPSs |
| `epm upgrade <pkg>` | Upgrade to the latest version |
| `epm uninstall <pkg>` | Remove an installed EPS |

## Registry Rules (enforced at publish time)

- `CUSTOMIZE.md` must exist (ADR-5)
- Package name must be flat, lowercase, `[a-z][a-z0-9_]{1,63}` (ADR-10)
- Declared platform targets must be Rust target triples (ADR-6)
- License must be MIT, Apache-2.0, BSD, or MPL (ADR-7)
- Version is immutable once published (ADR-7)

## Install Model

Installs are git-based and content-addressed (ADR-9). Each install pins to a specific
git commit SHA. The local store at `~/.epm/` caches fetched repos so repeat installs
are instant. See the `install_lifecycle` concept for the full end-to-end flow.
"#;

const EPC_CONCEPT: &str = r#"
# Concept: EPC (Extremely Personal Cloud)

EPC is the runtime layer of the EPS ecosystem. It deploys EPS packages as persistent
daemons on your own hardware, using Tailscale for networking. EPC is itself an EPS —
it dogfoods the format.

## The Pitch

All the UX of a PaaS (Heroku, Render, Fly.io), running entirely on your own hardware:
- No monthly bill
- No data leaving your network
- Reachable on all your devices via Tailscale

## Commands

| Command | What it does |
|---------|-------------|
| `epc deploy <spec>` | Install an EPS and start it as a persistent daemon |
| `epc ps` | List running services with their Tailscale URLs |
| `epc logs <name>` | Tail stdout/stderr for a service |
| `epc stop <name>` | Stop a running service |

## How it Works

EPC reads your Tailscale node name at startup. When you deploy a service, EPC:
1. Calls `epm install` to install the EPS
2. Starts the service using the `start` command from `eps.toml`'s `[service]` block
3. Records the service in `~/.epc/services.toml`
4. Surfaces the Tailscale URL (e.g. `http://my-node.tail.net:2248`) in `epc ps`

## EPS Service Declaration

EPSs that want to run as services declare it in `eps.toml`:

```toml
[service]
enabled = true
start   = "./run.sh serve"
port    = 2248
```

## Architecture

EPC is a Rust CLI (`main.rs, state.rs, tailscale.rs, eps.rs, commands/`). It uses either
a built-in tokio supervisor loop or integrates with launchd (macOS) / systemd (Linux)
for process management. The `dashboard` port (false by default) can expose a web UI
listing all running services with their URLs.

## Relationship to EPM

```
EPM installs EPSs
EPC runs EPSs as persistent services
EPS defines the format both tools understand
```
"#;

// ── Practical reference material ──────────────────────────────────────────────

pub const EPS_TOML_REFERENCE: &str = r#"
# `eps.toml` — Complete Annotated Schema

Every EPS must include `eps.toml` at its repository root. Required fields are marked.

```toml
# ── [package] — required ──────────────────────────────────────────────────────

[package]
name        = "my_package"           # (required) Package name.
                                     #   Rules: 2–64 chars, starts with [a-z],
                                     #   remaining chars [a-z0-9_]. No hyphens.
                                     #   Valid: "tech_talker", "pi", "todo"
                                     #   Invalid: "Tech-Talker", "my-pkg", "1st"

version     = "0.1.0"               # (required) Semver. Once published, immutable.

description = "One-line summary"    # (required) Shown in `epm search` results.

platform    = [                      # (required) Rust target triples.
  "aarch64-apple-darwin",           #   Apple Silicon macOS
  "x86_64-apple-darwin",            #   Intel macOS
  "x86_64-unknown-linux-gnu",       #   Linux x86_64
]                                    #   EPM hard-blocks install on platform mismatch.

authors     = ["Your Name"]         # optional
license     = "MIT"                 # optional — allowed: MIT, Apache-2.0, BSD, MPL
                                    #   GPL/AGPL shows a notice at install time.
                                    #   Proprietary/source-available blocks install.
homepage    = "https://..."         # optional
repository  = "https://..."         # optional — git URL EPM fetches from


# ── [eps] — required ──────────────────────────────────────────────────────────
# Presence of this block marks the repo as an EPS (not just any git repo).

[eps]
customization_guide = "CUSTOMIZE.md"  # optional, default: "CUSTOMIZE.md"
                                      # Path to the LLM-friendly customization doc.
hooks_dir           = "Scripts/"      # optional — directory containing hook scripts


# ── [hooks] — optional ────────────────────────────────────────────────────────
# Shell scripts EPM runs at lifecycle points. Paths relative to install dir.
# Scripts run in a clean environment with these vars set:
#   EPM_PACKAGE_NAME, EPM_PACKAGE_VERSION, EPM_INSTALL_DIR,
#   EPM_CACHE_DIR, EPM_PLATFORM

[hooks]
install   = "Scripts/install.sh"    # run after files are copied
configure = "Scripts/configure.sh"  # optional — post-install setup
update    = "Scripts/update.sh"     # optional — run on `epm upgrade`
uninstall = "Scripts/uninstall.sh"  # optional — run on `epm uninstall`


# ── [system-dependencies] — optional ─────────────────────────────────────────
# Tools EPM checks are present before building. EPM never auto-installs these —
# it hard-blocks with actionable install instructions if any are missing.

[system-dependencies]
brew = ["cmake", "libomp"]          # Homebrew packages (macOS)
gem  = ["xcpretty"]                 # RubyGems
apt  = ["build-essential", "curl"]  # apt packages (Linux)


# ── [service] — optional ──────────────────────────────────────────────────────
# Declares that this EPS can run as a persistent daemon via `epc deploy`.

[service]
enabled = true
start   = "./run.sh serve"          # command EPC uses to start the daemon
port    = 2248                      # port the service listens on
                                    # EPC surfaces this as the Tailscale URL
```

## Minimal valid eps.toml

```toml
[package]
name        = "my_package"
version     = "0.1.0"
description = "A minimal EPS harness"
platform    = ["aarch64-apple-darwin"]

[eps]
```

## Notes

- `epm init <name>` scaffolds a valid eps.toml pre-filled from `git config`
- The `[eps]` block must be present even if empty — it's what distinguishes an EPS from
  a regular repo
- `version` is immutable once published; bump it for every new publish
- `platform` controls install eligibility; omitting it is a publish error (ADR-6)
"#;

pub const CUSTOMIZE_MD_TEMPLATE: &str = r#"
# CUSTOMIZE.md — Fill-in-the-Blanks Template

Copy this into your EPS root and fill in each section. Every published EPS must have
this file. The registry checks for its existence at `epm publish` time.

---

```markdown
# <package-name> — Customization Guide

<One paragraph: what does this harness do and who is it for? What does it ship with
by default, and what is deliberately left for the user to customize?>

## Ports

<List every port. A port is a named, designed-in extension point — not a config value,
but a structural seam where behavior can be replaced. Each port gets its own section.>

### `PORT_NAME`

**What it does:** <One sentence describing what this extension point controls.>
**Default:** <What ships out of the box. Be specific — file name, function name, value.>
**How to customize:** <Exact instructions. What file to edit, what to change, what
format to use. An LLM reading this should be able to make the change without guessing.>

### `ANOTHER_PORT`

**What it does:** ...
**Default:** ...
**How to customize:** ...

## Getting Started

1. Clone the repo: `git clone <repository>`
2. <Any required setup — dependencies, build steps, config files>
3. Run: `./run.sh` (or whatever the entry point is)

## Common Customizations

<Optional but recommended: 2–3 worked examples showing the most common things users
will want to change. Show before/after. Be concrete.>

### Example: <Changing X to Y>

<Step-by-step walkthrough.>
```

---

## Rules for a good CUSTOMIZE.md

- **One file, one fetch.** Don't split across multiple files or link to a doc site.
  An LLM answering "how do I change X?" fetches this file and should find the answer.
- **Every port is listed.** If it's not in CUSTOMIZE.md, it's not a port — it's a
  hidden implementation detail.
- **Actionable, not descriptive.** "Edit `config/model.py` and replace the class name"
  is good. "The model is configurable" is not.
- **Ports, not settings.** A port changes behavior. A config key changes a value. Both
  can appear here, but ports are the product — lead with them.
"#;

pub fn get_example(name: &str) -> String {
    match name.to_lowercase().as_str() {
        "tech_talker" => EXAMPLE_TECH_TALKER.to_string(),
        "pi" => EXAMPLE_PI.to_string(),
        "todo" => EXAMPLE_TODO.to_string(),
        _ => format!(
            "Unknown example '{name}'. Valid options: tech_talker, pi, todo."
        ),
    }
}

const EXAMPLE_TECH_TALKER: &str = r#"
# Canonical Example: `tech_talker`

A macOS audio transcription harness. Ships with Whisper (local) and FluidAudio
(streaming) support, global hotkeys, and a model manager. The ports are the product —
the author chose not to hardcode the model, the hotkey, or what happens to the text.

## eps.toml

```toml
[package]
name        = "tech_talker"
version     = "1.0.0"
description = "Local-first audio transcription for macOS using Whisper"
authors     = ["nickagliano"]
license     = "MIT"
platform    = ["aarch64-apple-darwin", "x86_64-apple-darwin"]
repository  = "https://github.com/nickagliano/tech_talker"

[eps]
customization_guide = "CUSTOMIZE.md"
hooks_dir           = "Scripts/"

[hooks]
install   = "Scripts/install.sh"
uninstall = "Scripts/uninstall.sh"

[system-dependencies]
brew = ["cmake", "libomp"]
gem  = ["xcpretty"]
```

## CUSTOMIZE.md

```markdown
# tech_talker — Customization Guide

tech_talker is a macOS audio transcription harness. It listens for a global hotkey,
records audio while the key is held, and transcribes the recording when released.
What you do with the transcript — and which model transcribes it — is up to you.

## Ports

### `TRANSCRIPTION_ENGINE`

**What it does:** Controls how audio is transcribed. Two engines ship: `whisper`
(local, private, slower) and `fluid` (streaming via FluidAudio API, faster).
**Default:** `whisper`
**How to customize:** Edit `config/engine.toml` and set `engine = "fluid"`. For
FluidAudio, also set `fluid_api_key = "your-key"`.

### `WHISPER_MODEL`

**What it does:** Selects which Whisper model is loaded for local transcription.
Larger models are more accurate but slower and use more RAM.
**Default:** `whisper-base`
**How to customize:** Run `tech_talker model set <model-name>`. Available models:
`whisper-tiny`, `whisper-base`, `whisper-small`, `whisper-medium`, `whisper-large-v3`.

### `GLOBAL_HOTKEY`

**What it does:** The key combination that starts/stops recording system-wide.
**Default:** `Cmd+Shift+Space`
**How to customize:** Edit `config/hotkey.toml` and set `hotkey = "Cmd+Shift+X"`
(or any valid macOS key combo). Restart tech_talker for changes to take effect.

### `TRANSCRIPT_HANDLER`

**What it does:** The function called with the final transcript string. Default
behavior pastes the text at the cursor. Replace this to route transcripts elsewhere.
**Default:** Pastes text at cursor via Accessibility API
**How to customize:** Edit `Sources/TechTalker/TranscriptHandler.swift` and replace
the body of `handle(transcript:)`. Common replacements: append to a file, POST to a
webhook, pipe to another app.

## Getting Started

1. Clone: `git clone https://github.com/nickagliano/tech_talker`
2. Install system deps: `brew install cmake libomp && gem install xcpretty`
3. Run installer: `Scripts/install.sh`
4. Grant Accessibility and Microphone permissions when prompted
5. Hold `Cmd+Shift+Space` to record, release to transcribe
```

## Design notes

- Four ports, not forty settings. The author could have added config keys for every
  Whisper parameter — beam size, temperature, language — but chose not to. The four
  ports represent the four things users actually want to change.
- The TRANSCRIPT_HANDLER port is the most powerful: it's a code port (edit a Swift
  function) rather than a config port. This gives users full control at the cost of
  requiring them to write Swift.
- System dependencies are declared explicitly so EPM can check them before building.
  A missing `cmake` would cause a cryptic build failure; a missing dep in eps.toml
  causes a clear error with install instructions.
"#;

const EXAMPLE_PI: &str = r#"
# Canonical Example: `pi`

A minimal agent harness (powers OpenClaw, an open-source Claude Code alternative).
Ships with exactly 4 tools: read, write, edit, bash. Every other capability is a port.
This is the archetype of a framework harness: the extension surface is the product.

## eps.toml

```toml
[package]
name        = "pi"
version     = "0.1.0"
description = "A minimal agent harness. 4 tools. Everything else is a port."
authors     = ["nickagliano"]
license     = "MIT"
platform    = ["aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"]
repository  = "https://github.com/nickagliano/pi"

[eps]
customization_guide = "CUSTOMIZE.md"
hooks_dir           = "Scripts/"

[hooks]
install = "Scripts/install.sh"
```

## CUSTOMIZE.md

```markdown
# pi — Customization Guide

pi is a minimal agent harness. It ships with a read-eval loop, a system prompt, and
4 tools: read, write, edit, bash. The loop is the skeleton. Everything else is yours.

## Ports

### `TOOL_REGISTRY`

**What it does:** The set of tools available to the agent at runtime.
**Default:** `[read, write, edit, bash]` — defined in `src/tools/mod.rs`
**How to customize:** Add a new file in `src/tools/`, implement the `Tool` trait,
and register it in `src/tools/mod.rs` in the `all_tools()` function. The agent
sees every registered tool automatically.

### `SYSTEM_PROMPT`

**What it does:** The base instructions given to the model at the start of every
session. Controls persona, constraints, and default behavior.
**Default:** A minimal prompt describing the 4 available tools.
**How to customize:** Edit `prompts/system.md`. Markdown is supported. The file
is read at startup — no recompile needed.

### `SLASH_COMMANDS`

**What it does:** Short commands the user can type (e.g. `/commit`, `/review`)
that expand to full prompts or trigger side effects before the model sees them.
**Default:** None
**How to customize:** Add a file to `slash_commands/` named `<command>.md` (for
prompt expansion) or `<command>.rs` (for code execution). See `slash_commands/README.md`.

### `TUI_RENDERER`

**What it does:** The terminal UI — how the conversation, tool calls, and output
are displayed.
**Default:** Plain text with basic ANSI formatting
**How to customize:** Implement the `Renderer` trait in `src/tui/renderer.rs` and
swap the default in `src/main.rs`. A ratatui-based implementation is in `examples/`.

## Getting Started

1. Clone: `git clone https://github.com/nickagliano/pi`
2. Build: `cargo build --release`
3. Run: `./target/release/pi`
4. Type a message and press Enter.
```

## Design notes

- pi is the extreme case of a framework harness: it barely does anything by default.
  The 4 tools are the minimum to be useful (and to prove it works). The restraint is
  deliberate.
- SYSTEM_PROMPT is a file port, not a code port — change the markdown file, restart.
  No compilation for the most common customization.
- TOOL_REGISTRY is a code port — it requires writing Rust. The author chose this
  because tools are structural, not just configurable.
- Slash commands support both prompt expansion (md files) and code execution (rs files),
  covering the full spectrum from simple to powerful.
"#;

const EXAMPLE_TODO: &str = r#"
# Canonical Example: `todo`

A local task + reading list manager with a web UI, served over HTTP. Ships with JSON
storage and plain text output. Demonstrates ports-as-traits (Storage, Formatter) and
a lifecycle hook port — shell scripts that fire on add/complete/delete without touching
Rust. Deployed as a service harness via EPC on port 8765.

## eps.toml

```toml
[package]
name        = "todo"
version     = "0.1.3"
description = "A minimal CLI todo list harness — storage, output, and hooks are all ports."
authors     = ["nickagliano"]
license     = "MIT"
repository  = "https://github.com/nickagliano/todo"
platform    = ["aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"]

[eps]
customization_guide = "CUSTOMIZE.md"
hooks_dir           = "hooks/"

[service]
enabled = true
start   = "HOST=$(tailscale ip -4) ./serve.sh"
port    = 8765
```

## CUSTOMIZE.md

```markdown
# todo — Customization Guide

todo is a local task and reading list manager. It ships with JSON file storage,
plain-text output, and a web UI served over HTTP. The storage backend, output
formatter, lifecycle hooks, and reading list are all ports.

## Ports

### `STORAGE_BACKEND`

**What it does:** Controls where tasks are persisted.
**Default:** `JsonStorage` — writes a JSON array to `~/.config/simple_todo/tasks.json`
**How to customize:** Implement the `Storage` trait in `src/storage.rs`:

  ```rust
  pub trait Storage {
      fn load(&self) -> Result<Vec<Task>>;
      fn save(&self, tasks: &[Task]) -> Result<()>;
  }
  ```

  Then swap in `main.rs`:
  ```rust
  let storage = SqliteStorage::new("~/.config/todo/tasks.db")?; // was: JsonStorage::default()
  ```

  Example replacements: `SqliteStorage`, `HttpStorage`, `EncryptedStorage`, `MultiStorage`.

### `OUTPUT_FORMATTER`

**What it does:** Controls how tasks are rendered on `list`.
**Default:** `PlainText` — numbered checklist
**How to customize:** Implement the `Formatter` trait in `src/formatter.rs`:

  ```rust
  pub trait Formatter {
      fn format(&self, tasks: &[Task]) -> String;
  }
  ```

  Example replacements: `TableFormatter`, `MarkdownFormatter`, `JsonFormatter`, `PriorityFormatter`.

### `LIFECYCLE_HOOKS`

**What it does:** Shell scripts that fire after add / complete / delete operations.
**Default:** Three stub scripts in `hooks/` that do nothing.
**How to customize:** Edit `hooks/on_add.sh`, `hooks/on_complete.sh`, `hooks/on_delete.sh`.
  Each receives `$1 = task id`, `$2 = task text`. No Rust required.

  Example uses: macOS notifications via `osascript`, logging completions to markdown,
  POST to a webhook, sync to another tool's CLI.

### `READING_LIST`

**What it does:** Controls where reading list items are stored (a secondary list
alongside tasks — URLs, book titles, anything to track separately).
**Default:** `JsonReadingList` — JSON array at `~/.config/simple_todo/reading.json`
**How to customize:** Implement the `ReadingList` trait in `src/reading.rs`:

  ```rust
  pub trait ReadingList {
      fn load(&self) -> Result<Vec<ReadingItem>>;
      fn save(&self, items: &[ReadingItem]) -> Result<()>;
  }
  ```

  Example swaps: `SqliteReadingList`, `PocketSync`, `BookmarksImport`.

## Getting Started

1. Clone: `git clone https://github.com/nickagliano/todo`
2. Build: `cargo build --release`
3. Run: `./serve.sh` (or deploy via EPC)
```

## Design notes

- Ports-as-traits is the strongest form of port: the Rust type system enforces the
  contract. You can't half-implement a Storage backend — it won't compile.
- LIFECYCLE_HOOKS is a plugin port (directory scanning) — the highest-friction, most
  powerful form. Shell scripts mean zero recompilation for behavioral changes.
- todo is a service harness: it binds port 8765 and EPC manages it as a persistent
  daemon. The web UI is accessible from any device on the tailnet.
- What's deliberately missing: priorities/tags, due dates, recurring tasks, multiple
  lists, undo. These are ports, not bugs — extend Task or add storage keys.
"#;

const OBSERVATORY_CONCEPT: &str = r#"
# Concept: Observatory

Observatory is a lightweight health monitoring EPS for EPC clusters. It replaces
heavy stacks like Grafana + Prometheus + OpenTelemetry with a lean Rust service
that requires no external databases or containers.

## What it does

- Polls the health endpoint of every service in `~/.epc/services.toml` on a configurable
  interval (default: 30s)
- Stores results in SQLite at `~/.epc/observatory.db`
- Serves a mobile-friendly dark-mode dashboard at port 9090 with:
  - Status badge per service (🟢 running / 🟡 degraded / 🔴 stopped)
  - Last response time in ms
  - Dot-grid sparkline of the last 40 checks (● running, ◐ degraded, ○ stopped)
  - Auto-refreshes every 30s
- Fires txtme alerts on state transitions (running → degraded, stopped → running, etc.)

## Service Discovery

Observatory reads `~/.epc/services.toml` (EPC's state file) to discover managed services,
then reads each service's `eps.toml` to check if a `health_check` is declared. If yes,
it GETs `http://<tailscale-ip>:<port>/health`. If no health_check is declared, it falls
back to a port-listening check via `lsof`.

## Deployment

```bash
cd observatory
cp .env.example .env        # set TXTME_URL
cargo build --release
epc deploy --local ./       # deploys as EPS at port 9090
```

Or run directly: `./serve.sh`

## Env Vars

| Var | Default | Purpose |
|-----|---------|---------|
| `HOST` | `127.0.0.1` | Bind address (EPS start command sets this to Tailscale IP) |
| `PORT` | `9090` | Web server port |
| `TXTME_URL` | — | txtme endpoint for SMS alerts on state transitions |
| `POLL_INTERVAL_SECS` | `30` | Health check frequency |

## Routes

- `GET /` — mobile-friendly HTML dashboard
- `GET /health` — `ok` (so EPC can health-check Observatory itself)
- `GET /api/services` — JSON snapshot of current service states

## eps.toml

```toml
[package]
name        = "observatory"
version     = "0.1.0"
description = "Lightweight health monitoring dashboard for EPC services"
platform    = ["aarch64-apple-darwin"]

[service]
enabled      = true
start        = "HOST=$(tailscale ip -4) ./serve.sh"
port         = 9090
health_check = "GET /health"
```

## SQLite Schema

```sql
health_checks(id, service, checked_at, status, response_ms, status_code)
service_state(service, last_status, last_checked)
```

## Ports (extension points)

- `POLL_INTERVAL_SECS` — env var
- `TXTME_URL` — env var, enables SMS alerts
- `DASHBOARD_SPARKLINE_LENGTH` — edit `src/dashboard.rs` (default 40 dots)
- Custom alert logic — edit `src/poller.rs` `poll_once()` for threshold-based alerting
- Additional non-EPC services — hardcode extra entries in `src/poller.rs`
"#;

// ── Concept explanations (existing) ───────────────────────────────────────────

const SEASONINGS_CONCEPT: &str = r#"
# Concept: EPS Seasonings

Seasonings are portable, LLM-ready implementation patterns for EPS apps. Each seasoning
is a self-contained markdown file describing a single capability — what it does, why it
works, and exactly how to apply it.

**Registry:** https://github.com/nickagliano/eps_seasonings

Seasonings live in `seasonings/` as individual markdown files. Fetch the raw file and
tell Claude to apply it to the target project.
"#;

const INSTALL_LIFECYCLE_CONCEPT: &str = r#"
# Concept: Install Lifecycle

`epm install` is a multi-stage pipeline. Each stage has its own caching layer so
repeated installs of the same version are instant. (ADR-9)

## Directory Layout

```
~/.epm/
  cache/
    git/<host>/<owner>/<repo>/<commit-sha>/   # bare git clones
    builds/<source-hash>-<platform>/          # cached build artifacts
  installs/
    <package-name>/<version>/                 # active installs
  state.toml                                  # installed package registry
```

## Stages

### 1. Resolution
- Query registry for metadata (git URL, commit SHA, platforms, license)
- If no version specified: resolve to latest published
- **Platform check** (ADR-6): hard-block on mismatch unless `--allow-unsupported-platform`
- **License check** (ADR-7): GPL/AGPL shows a notice; proprietary/source-available blocks

### 2. Fetch
- Cache key: `(git_url, commit_sha)`
- If cache hit: skip entirely (instant)
- Otherwise: `git clone --depth 1` at the resolved commit SHA

The git SHA is both identifier and integrity check — content-addressed by nature.

### 3. System Dependency Check (ADR-13)
- If `eps.toml` has `[system-dependencies]`: check each declared tool
- Hard-block with actionable error if any are missing:
  ```
  error: missing system dependencies — run:
    brew install cmake
    brew install libomp
    gem install xcpretty
  ```
- EPM never auto-installs system deps (side effects require explicit user consent)

### 4. Build (optional)
- If `eps.toml` declares a `build` hook:
- Cache key: `(source_hash, platform_triple)`
- Cache hit: skip build
- Otherwise: run the build hook, cache artifacts

### 5. Install
- Source: `~/.epm/cache/git/.../<commit-sha>/`
- Destination: `~/.epm/installs/<name>/<version>/`
- Copy strategy (preferred → fallback):
  1. APFS clonefiles (macOS) — zero-cost copy-on-write
  2. Hardlinks — no data duplication
  3. Full copy — fallback

### 6. Hook Execution
- Run `install` hook from `eps.toml` with working dir = install dir
- Environment variables passed: `EPM_PACKAGE_NAME`, `EPM_PACKAGE_VERSION`,
  `EPM_INSTALL_DIR`, `EPM_CACHE_DIR`, `EPM_PLATFORM`
- Clean environment (does not inherit user's shell env)

### 7. State Recording
- Record in `~/.epm/state.toml`:
  ```toml
  [[installed]]
  name        = "tech_talker"
  version     = "1.0.0"
  platform    = "aarch64-apple-darwin"
  source_sha  = "abc123..."
  install_dir = "/Users/nick/.epm/installs/tech_talker/1.0.0"
  installed_at = "2026-02-26T14:00:00Z"
  ```
- This is the source of truth for `epm list`, `epm upgrade`, `epm uninstall`
"#;
