#[tokio::main]
async fn main() -> anyhow::Result<()> {
    noor_notes::managed_app::run().await?;
    Ok(())
}
