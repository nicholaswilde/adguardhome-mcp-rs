#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    adguardhome_mcp_rs::run(std::env::args().collect()).await
}
