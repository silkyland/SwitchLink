mod gui;
mod cli;
mod file_manager;
mod usb;
mod protocol;

fn main() {
    // Initialize logging with DEBUG level to see detailed protocol communication
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Check if running in CLI mode
    let args: Vec<String> = std::env::args().collect();

    if args.contains(&"--cli".to_string()) {
        // Run CLI version
        cli::run_cli();
    } else {
        // Launch the eGUI
        println!("🎮 Launching DBI Backend with eGUI...");
        println!("✅ eGUI works perfectly in Linux!");
        gui::launch_gui();
    }
}
