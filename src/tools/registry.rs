// Persona component registry tools.
//
// Talks to the persona_registry service (default: http://100.78.103.79:3004).
// The registry stores reusable components (template.html, style.css, script.js)
// that can be installed into any persona site's components/ directory.

use serde::{Deserialize, Serialize};
use std::fmt::Write;

fn registry_base() -> String {
    std::env::var("PERSONA_REGISTRY_URL")
        .unwrap_or_else(|_| "http://100.78.103.79:3004".to_string())
}

#[derive(Debug, Deserialize, Serialize)]
struct Component {
    id: i64,
    name: String,
    description: Option<String>,
    author: Option<String>,
    tags: Option<String>,
    version: String,
    created_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ComponentFile {
    id: i64,
    component_id: i64,
    filename: String,
    content: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ComponentWithFiles {
    id: i64,
    name: String,
    description: Option<String>,
    author: Option<String>,
    tags: Option<String>,
    version: String,
    created_at: String,
    files: Vec<ComponentFile>,
}

fn format_component(c: &Component) -> String {
    let mut out = String::new();
    let _ = write!(out, "**{}** v{}", c.name, c.version);
    if let Some(desc) = &c.description {
        let _ = write!(out, " — {desc}");
    }
    if let Some(tags) = &c.tags {
        let _ = write!(out, "\n  tags: {tags}");
    }
    if let Some(author) = &c.author {
        let _ = write!(out, "\n  author: {author}");
    }
    let _ = write!(out, "\n  usage: `{{{{{}}}}}`", c.name);
    out
}

// ── list_persona_components ───────────────────────────────────────────────────

pub fn list_components() -> String {
    let url = format!("{}/api/components", registry_base());
    match ureq::get(&url).call() {
        Ok(resp) => {
            let body = resp.into_string().unwrap_or_default();
            match serde_json::from_str::<Vec<Component>>(&body) {
                Ok(components) if components.is_empty() => {
                    "The persona registry is empty. No components published yet.".to_string()
                }
                Ok(components) => {
                    let mut out =
                        format!("# Persona Registry — {} components\n\n", components.len());
                    for c in &components {
                        out.push_str(&format_component(c));
                        out.push_str("\n\n");
                    }
                    out.push_str(&format!(
                        "---\nBrowse at: {}\n\
                         Install with: install_persona_component(name, persona_site_path)",
                        registry_base()
                    ));
                    out
                }
                Err(e) => format!("Failed to parse component list: {e}\nBody: {body}"),
            }
        }
        Err(e) => format!(
            "Could not reach persona registry at {url}.\n\
             Is persona_registry running? Error: {e}"
        ),
    }
}

// ── search_persona_components ─────────────────────────────────────────────────

pub fn search_components(query: &str) -> String {
    let url = format!("{}/api/components?q={}", registry_base(), query);
    match ureq::get(&url).call() {
        Ok(resp) => {
            let body = resp.into_string().unwrap_or_default();
            match serde_json::from_str::<Vec<Component>>(&body) {
                Ok(components) if components.is_empty() => {
                    format!("No components found matching \"{query}\".")
                }
                Ok(components) => {
                    let mut out = format!(
                        "# Persona Registry — {} results for \"{query}\"\n\n",
                        components.len()
                    );
                    for c in &components {
                        out.push_str(&format_component(c));
                        out.push_str("\n\n");
                    }
                    out
                }
                Err(e) => format!("Failed to parse search results: {e}\nBody: {body}"),
            }
        }
        Err(e) => format!(
            "Could not reach persona registry at {url}.\nError: {e}"
        ),
    }
}

// ── get_persona_component ─────────────────────────────────────────────────────

pub fn get_component(name: &str) -> String {
    let url = format!("{}/api/components/{name}", registry_base());
    match ureq::get(&url).call() {
        Ok(resp) => {
            let body = resp.into_string().unwrap_or_default();
            match serde_json::from_str::<ComponentWithFiles>(&body) {
                Ok(c) => {
                    let mut out = format!("# Component: {}\n\n", c.name);
                    if let Some(desc) = &c.description {
                        out.push_str(&format!("{desc}\n\n"));
                    }
                    out.push_str(&format!("**Version:** {}\n", c.version));
                    if let Some(author) = &c.author {
                        out.push_str(&format!("**Author:** {author}\n"));
                    }
                    if let Some(tags) = &c.tags {
                        out.push_str(&format!("**Tags:** {tags}\n"));
                    }
                    out.push_str(&format!("\n**Usage:** `{{{{{}}}}}`\n\n", c.name));
                    out.push_str(&format!(
                        "**Files:** {}\n\n",
                        c.files.iter().map(|f| f.filename.as_str()).collect::<Vec<_>>().join(", ")
                    ));
                    for file in &c.files {
                        out.push_str(&format!(
                            "### {}\n```\n{}\n```\n\n",
                            file.filename, file.content
                        ));
                    }
                    out.push_str(&format!(
                        "---\nTo install: call install_persona_component(\"{}\", \"/path/to/persona\")",
                        c.name
                    ));
                    out
                }
                Err(e) => format!("Failed to parse component \"{name}\": {e}\nBody: {body}"),
            }
        }
        Err(ureq::Error::Status(404, _)) => {
            format!("Component \"{name}\" not found in the registry.\nUse list_persona_components() to see what's available.")
        }
        Err(e) => format!(
            "Could not reach persona registry at {url}.\nError: {e}"
        ),
    }
}

// ── install_persona_component ─────────────────────────────────────────────────

pub fn install_component(name: &str, persona_path: &str) -> String {
    let url = format!("{}/api/components/{name}", registry_base());
    let c = match ureq::get(&url).call() {
        Ok(resp) => {
            let body = resp.into_string().unwrap_or_default();
            match serde_json::from_str::<ComponentWithFiles>(&body) {
                Ok(c) => c,
                Err(e) => return format!("Failed to parse component \"{name}\": {e}\nBody: {body}"),
            }
        }
        Err(ureq::Error::Status(404, _)) => {
            return format!(
                "Component \"{name}\" not found in registry.\n\
                 Use list_persona_components() to see available components."
            );
        }
        Err(e) => {
            return format!("Could not reach persona registry at {url}.\nError: {e}");
        }
    };

    let component_dir = std::path::Path::new(persona_path)
        .join("components")
        .join(&c.name);

    if let Err(e) = std::fs::create_dir_all(&component_dir) {
        return format!(
            "Failed to create directory {}: {e}",
            component_dir.display()
        );
    }

    let mut written = Vec::new();
    for file in &c.files {
        let path = component_dir.join(&file.filename);
        if let Err(e) = std::fs::write(&path, &file.content) {
            return format!("Failed to write {}: {e}", path.display());
        }
        written.push(file.filename.clone());
    }

    format!(
        "✓ Installed `{name}` v{version} into {dir}\n\
         Files written: {files}\n\n\
         Next step: run `POST /admin/rebuild` (or hit Rebuild in the admin) to register the component.\n\
         Then use `{{{{{name}}}}}` in any page's markdown.",
        version = c.version,
        dir = component_dir.display(),
        files = written.join(", "),
    )
}

// ── publish_persona_component ─────────────────────────────────────────────────

pub fn publish_component(
    persona_path: &str,
    name: &str,
    description: Option<&str>,
    author: Option<&str>,
    tags: Option<&str>,
    token: Option<&str>,
) -> String {
    let component_dir = std::path::Path::new(persona_path)
        .join("components")
        .join(name);

    if !component_dir.exists() {
        return format!(
            "Component directory not found: {}\n\
             Make sure the component exists in the persona site before publishing.",
            component_dir.display()
        );
    }

    let filenames = ["template.html", "style.css", "script.js"];
    let mut files = Vec::new();

    for fname in &filenames {
        let path = component_dir.join(fname);
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => files.push(serde_json::json!({
                    "filename": fname,
                    "content": content
                })),
                Err(e) => return format!("Failed to read {}: {e}", path.display()),
            }
        }
    }

    if files.is_empty() {
        return format!(
            "No files found in {}\n\
             Expected at least template.html.",
            component_dir.display()
        );
    }

    let auth_token = token
        .map(|t| t.to_string())
        .or_else(|| std::env::var("REGISTRY_TOKEN").ok())
        .unwrap_or_else(|| "dev-token".to_string());

    let payload = serde_json::json!({
        "name": name,
        "description": description,
        "author": author,
        "tags": tags,
        "version": "1.0.0",
        "files": files
    });

    let url = format!("{}/api/components", registry_base());
    match ureq::post(&url)
        .set("Authorization", &format!("Bearer {auth_token}"))
        .set("Content-Type", "application/json")
        .send_string(&payload.to_string())
    {
        Ok(_) => format!(
            "✓ Published `{name}` to the persona registry.\n\
             View at: {}/components/{name}\n\
             Others can install it with: install_persona_component(\"{name}\", \"/path/to/persona\")",
            registry_base()
        ),
        Err(e) => format!("Failed to publish component \"{name}\": {e}"),
    }
}
