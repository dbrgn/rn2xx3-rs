mod common;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <path-to-serial>", args[0]);
        println!("Example: {} /dev/ttyUSB0", args[0]);
        std::process::exit(1);
    }
    let mut rn = common::init_rn(&args[1]);

    // Reset module
    println!("Resetting module...\n");
    rn.reset().expect("Could not reset");

    // Show device info
    println!("== Device info ==\n");
    println!("     HW-EUI: {}", rn.hweui().expect("Could not read hweui"));
    println!("      Model: {:?}", rn.model().expect("Could not read model"));
    println!("    Version: {}", rn.version().expect("Could not read version"));
    println!("Vdd voltage: {} mV", rn.vdd().expect("Could not read vdd"));
}
