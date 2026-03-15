use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod knowledge;
mod registry;

// ── Server ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct EpsMcp {
    tool_router: ToolRouter<Self>,
}

impl EpsMcp {
    pub fn new() -> Self {
        Self { tool_router: Self::tool_router() }
    }
}

#[tool_handler]
impl ServerHandler for EpsMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "This server provides authoritative knowledge about the Extremely Personal \
                 Software (EPS) ecosystem — the philosophy, ADRs, package manager (EPM), \
                 runtime (EPC), and canonical examples. Call get_overview() first to \
                 understand the ecosystem, then use get_adr(number) to read specific design \
                 decisions, and get_concept(name) to deep-dive into core ideas. Use this \
                 knowledge to help design, build, and customize EPS harnesses. \
                 For EPC-specific documentation (commands, architecture, quickstart, etc.) \
                 call list_epc_docs() to see what's available, then get_epc_doc(path) to \
                 read any file — these are served live from the epc repo so they are always \
                 up to date."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

// ── Server info update ────────────────────────────────────────────────────────

// (server info includes persona registry tools — see below)

// ── Tool parameter types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetAdrParams {
    /// ADR number (1–14). Returns the full text of that Architecture Decision Record.
    pub number: u32,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetConceptParams {
    /// Concept name. One of: "eps", "epm", "epc", "port", "harness",
    /// "customize_md", "litmus_test", "install_lifecycle", "observatory".
    pub concept: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetExampleParams {
    /// Name of the canonical EPS to show. One of: "tech_talker", "pi", "todo".
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchPersonaComponentsParams {
    /// Search query. Matches against component name, description, and tags.
    pub query: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetPersonaComponentParams {
    /// Component name, e.g. "music", "frame", "now-playing".
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct InstallPersonaComponentParams {
    /// Component name to install from the registry, e.g. "now-playing".
    pub name: String,
    /// Absolute path to the persona site directory, e.g. "/Users/nick/Documents/personal-projects/persona".
    pub persona_path: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct PublishPersonaComponentParams {
    /// Absolute path to the persona site directory containing the component.
    pub persona_path: String,
    /// Component name (must match the directory name in components/).
    pub name: String,
    /// Short description of what the component does.
    pub description: Option<String>,
    /// Author name or handle.
    pub author: Option<String>,
    /// Comma-separated tags, e.g. "music,audio,player".
    pub tags: Option<String>,
    /// Registry auth token. Falls back to REGISTRY_TOKEN env var, then "dev-token".
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct GetEpcDocParams {
    /// Relative path to the doc file, e.g. "users/quickstart.md" or
    /// "developers/architecture.md". Call list_epc_docs() first to see all available paths.
    pub path: String,
}

// ── Tools ─────────────────────────────────────────────────────────────────────

#[tool_router]
impl EpsMcp {
    /// Returns a high-level overview of the entire EPS ecosystem:
    /// what EPS/EPM/EPC are and how they relate.
    #[tool(description = "Get a high-level overview of the Extremely Personal Software (EPS) ecosystem.")]
    async fn get_overview(&self) -> String {
        knowledge::OVERVIEW.to_string()
    }

    /// Returns the full text of an Architecture Decision Record by number (1–12).
    #[tool(description = "Get an Architecture Decision Record (ADR) by number. ADRs 1–14 cover all core EPS design decisions.")]
    async fn get_adr(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<GetAdrParams>,
    ) -> String {
        knowledge::get_adr(p.number)
    }

    /// Lists all ADRs with their numbers and one-line summaries.
    #[tool(description = "List all EPS Architecture Decision Records with their numbers and summaries.")]
    async fn list_adrs(&self) -> String {
        knowledge::ADR_INDEX.to_string()
    }

    /// Returns a detailed explanation of a core EPS concept.
    #[tool(description = "Get a detailed explanation of a core EPS concept: eps, epm, epc, port, harness, customize_md, litmus_test, install_lifecycle, or observatory.")]
    async fn get_concept(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<GetConceptParams>,
    ) -> String {
        knowledge::get_concept(&p.concept)
    }

    /// Returns the complete annotated eps.toml schema with all fields, required vs
    /// optional, valid values, and a minimal valid example.
    #[tool(description = "Get the complete annotated eps.toml schema — all fields, required vs optional, valid values, and a minimal valid example.")]
    async fn get_eps_toml_reference(&self) -> String {
        knowledge::EPS_TOML_REFERENCE.to_string()
    }

    /// Returns a fill-in-the-blanks CUSTOMIZE.md template with structural guidance
    /// and rules for writing a good customization guide.
    #[tool(description = "Get a fill-in-the-blanks CUSTOMIZE.md template with structural guidance and rules for what makes a good customization guide.")]
    async fn get_customize_md_template(&self) -> String {
        knowledge::CUSTOMIZE_MD_TEMPLATE.to_string()
    }

    /// Returns the eps.toml, CUSTOMIZE.md, and design notes for a canonical EPS example.
    #[tool(description = "Get the eps.toml, CUSTOMIZE.md, and design notes for a canonical EPS example. Valid names: tech_talker, pi, todo.")]
    async fn get_example(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<GetExampleParams>,
    ) -> String {
        knowledge::get_example(&p.name)
    }

    /// Returns an index of all EPC documentation files available in the epc repo.
    #[tool(description = "List all EPC documentation files (users guides, developer docs, ADRs). Returns paths you can pass to get_epc_doc. Call this before get_epc_doc to discover what's available.")]
    async fn list_epc_docs(&self) -> String {
        knowledge::list_epc_docs()
    }

    /// Reads a specific EPC documentation file from the epc repo by relative path.
    #[tool(description = "Read a specific EPC documentation file by relative path (e.g. \"users/quickstart.md\", \"developers/architecture.md\"). Call list_epc_docs() first to see all available paths.")]
    async fn get_epc_doc(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<GetEpcDocParams>,
    ) -> String {
        knowledge::get_epc_doc(&p.path)
    }

    // ── Persona Component Registry ─────────────────────────────────────────────

    /// Lists all components published to the Persona component registry.
    /// Returns name, description, version, tags, and usage shortcode for each.
    /// Use this to discover what's available before installing.
    #[tool(description = "List all components in the Persona component registry. Returns name, description, tags, and usage shortcode for each. Call this first to discover what's available.")]
    async fn list_persona_components(&self) -> String {
        registry::list_components()
    }

    /// Searches the Persona component registry by name, description, or tags.
    #[tool(description = "Search the Persona component registry. Matches against name, description, and tags. Returns matching components with usage instructions.")]
    async fn search_persona_components(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<SearchPersonaComponentsParams>,
    ) -> String {
        registry::search_components(&p.query)
    }

    /// Gets full details and file contents for a single Persona component.
    #[tool(description = "Get full details for a Persona component by name, including all file contents (template.html, style.css, script.js) and usage instructions.")]
    async fn get_persona_component(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<GetPersonaComponentParams>,
    ) -> String {
        registry::get_component(&p.name)
    }

    /// Downloads a component from the registry and installs it into a local Persona site.
    /// Writes files to {persona_path}/components/{name}/. After installing, trigger a rebuild.
    #[tool(description = "Install a Persona component from the registry into a local persona site. Downloads and writes all component files (template.html, style.css, script.js) to the site's components/ directory. After installing, run POST /admin/rebuild or hit Rebuild in the admin UI.")]
    async fn install_persona_component(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<InstallPersonaComponentParams>,
    ) -> String {
        registry::install_component(&p.name, &p.persona_path)
    }

    /// Publishes a component from a local Persona site to the registry so others can use it.
    /// Reads template.html, style.css, and script.js from {persona_path}/components/{name}/.
    #[tool(description = "Publish a Persona component to the shared registry. Reads the component files from a local persona site and uploads them so others can discover and install the component.")]
    async fn publish_persona_component(
        &self,
        rmcp::handler::server::wrapper::Parameters(p): rmcp::handler::server::wrapper::Parameters<PublishPersonaComponentParams>,
    ) -> String {
        registry::publish_component(
            &p.persona_path,
            &p.name,
            p.description.as_deref(),
            p.author.as_deref(),
            p.tags.as_deref(),
            p.token.as_deref(),
        )
    }
}
