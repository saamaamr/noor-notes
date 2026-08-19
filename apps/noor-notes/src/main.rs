#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
                "{} {}\nPrivate, offline-first notes for Linux\n\nUsage: {} [OPTION]\n\nOptions:\n  -h, --help       Show this help\n  -V, --version    Show the application version",
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
