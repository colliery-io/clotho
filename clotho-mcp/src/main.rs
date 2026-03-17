use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    clotho_mcp_server::run().await
}
