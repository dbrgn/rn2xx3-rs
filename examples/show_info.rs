mod common;

use std::env;

use env_logger;

fn main() {
    env_logger::init();

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
    let hweui = rn.hweui().expect("Could not read hweui");
    println!("     HW-EUI: {}", hweui);
    let model = rn.model().expect("Could not read model");
    println!("      Model: {:?}", model);
    let version = rn.version().expect("Could not read version");
    println!("    Version: {}", version);
    let vdd = rn.vdd().expect("Could not read vdd");
    println!("Vdd voltage: {} mV", vdd);
}
