use rusb::{Context, UsbContext};

fn main() {
    println!("Testing USB device detection...");
    println!("Looking for Nintendo Switch devices...");

    match Context::new() {
        Ok(context) => {
            match context.devices() {
                Ok(devices) => {
                    println!("Found {} USB devices", devices.len());
                    println!();

                    let mut switch_found = false;

                    for device in devices.iter() {
                        match device.device_descriptor() {
                            Ok(descriptor) => {
                                let vid = descriptor.vendor_id();
                                let pid = descriptor.product_id();
                                println!("VID: 0x{:04X}, PID: 0x{:04X}", vid, pid);

                                // Check for Nintendo Switch devices
                                if vid == 0x057E {
                                    switch_found = true;
                                    match pid {
                                        0x2000 => println!("  -> Nintendo Switch in NORMAL mode"),
                                        0x3000 => {
                                            println!("  -> Nintendo Switch in DBI mode (CORRECT)")
                                        }
                                        0x201D => println!("  -> Nintendo Switch in MTP mode"),
                                        _ => println!("  -> Nintendo Switch with unknown mode"),
                                    }
                                }
                            }
                            Err(e) => {
                                println!("Error getting device descriptor: {}", e);
                            }
                        }
                    }

                    println!();
                    if switch_found {
                        println!("✅ Nintendo Switch detected!");
                        println!("💡 Make sure to launch DBI on your Switch and select 'Run MTP responder' or 'Install title from DBIbackend'");
                    } else {
                        println!("❌ Nintendo Switch not found");
                        println!("🔧 Troubleshooting steps:");
                        println!("   1. Make sure your Switch is connected via USB");
                        println!("   2. Launch DBI on your Switch");
                        println!(
                            "   3. Select 'Run MTP responder' or 'Install title from DBIbackend'"
                        );
                    }
                }
                Err(e) => {
                    println!("Error enumerating devices: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error creating USB context: {}", e);
        }
    }
}
