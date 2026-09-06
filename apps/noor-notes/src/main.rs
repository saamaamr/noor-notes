#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // A headless diagnostic for packagers: validate the same configuration the
    // real account controller consumes, without opening notes or a keyring.
    if std::env::args().nth(1).as_deref() == Some("--check-cloud-config") {
        let configuration = noor_notes::cloud_config::CloudConfig::load()
            .map_err(|error| anyhow::anyhow!("Cloud configuration: {error}"))?;
        configuration
            .client()
            .map_err(|_| anyhow::anyhow!("Cloud configuration: client initialization failed"))?;
        println!("Cloud configuration valid; live sign-in and sync are not tested by this check");
        return Ok(());
    }
    if print_cli_information() {
        return Ok(());
    }
    noor_notes::managed_app::run().await?;
    Ok(())
}

fn print_cli_information() -> bool {
    match std::env::args().nth(1).as_deref() {
        Some("-h" | "--help") => {
            println!(
                "{} {}\nPrivate, offline-first notes for Linux\n\nUsage: {} [OPTION]\n\nOptions:\n  -h, --help             Show this help\n  -V, --version          Show the application version\n  --check-cloud-config   Validate account configuration without opening notes",
                noor_notes::identity::display_name(),
                env!("CARGO_PKG_VERSION"),
                noor_notes::identity::executable_name(),
            );
            true
        }
        Some("-V" | "--version") => {
            println!(
                "{} {}",
                noor_notes::identity::display_name(),
                env!("CARGO_PKG_VERSION")
            );
            true
        }
        _ => false,
    }
}
