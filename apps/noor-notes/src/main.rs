#[tokio::main]
async fn main() -> anyhow::Result<()> {
    noor_notes::app::run().await?;
    Ok(())
}
