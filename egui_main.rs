/// eGUI Version - Perfect for Linux!
mod egui_demo;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🦀 DBI Backend - eGUI Edition");
    println!("==============================");
    println!();
    println!("✅ eGUI provides:");
    println!("  • Native Linux support (no webkit)");
    println!("  • Immediate mode GUI (fast & responsive)");
    println!("  • Small binary size");
    println!("  • Easy to develop");
    println!("  • Great performance");
    println!();
    println!("🚀 Launching eGUI interface...");
    println!();

    egui_demo::launch_egui_demo();
}
