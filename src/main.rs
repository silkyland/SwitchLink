mod cli;
mod database;
mod file_manager;
mod gui;
mod protocol;
mod usb;

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
        println!("✅ Works on both Linux and Windows!");
        println!("💡 If you're on Windows and having connection issues:");
        println!("   1. Make sure you've installed libusb drivers");
        println!("   2. Use a data USB cable (not just a charging cable)");
        println!("   3. Try running as Administrator");
        println!("   4. Check that DBI is running on your Switch in the correct mode");
        gui::launch_gui();
    }
}
