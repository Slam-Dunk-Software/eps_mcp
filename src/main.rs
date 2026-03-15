mod tools;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tools::EpsMcp;

#[tokio::main]
async fn main() -> Result<()> {
    let server = EpsMcp::new();
    let transport = stdio();
    let service = server.serve(transport).await?;
    service.waiting().await?;
    Ok(())
}
