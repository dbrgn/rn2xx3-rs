mod common;

use std::env;
use std::io::{stdout, Write};

use rn2xx3::{ConfirmationMode, JoinMode};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        println!("Join the network via OTAA and send an uplink.");
        println!(
            "Usage: {} <path-to-serial> <appeui> <appkey> <port> <hexdata>",
            args[0]
        );
        println!(
            "Example: {} /dev/ttyUSB0 70B3D57ED000XXXX 120EEXXXXXXXXXXXXXXXXXXXXXXXXXXX 10 2342",
            args[0]
        );
        std::process::exit(1);
    }
    let mut rn = common::init_rn(&args[1]);

    // Reset module
    println!("Resetting module...");
    rn.reset().expect("Could not reset");

    // Set keys
    println!("Setting keys...");
    rn.set_app_eui_hex(&args[2]).expect("Could not set app EUI");
    rn.set_app_key_hex(&args[3]).expect("Could not set app key");

    // Join
    print!("Joining via OTAA...");
    stdout().flush().expect("Could not flush stdout");
    rn.join(JoinMode::Otaa).expect("Could not join");
    println!("OK");

    // Send data
    let port: u8 = args[4].parse().expect("Invalid port");
    rn.transmit_hex(ConfirmationMode::Unconfirmed, port, &args[5])
        .expect("Could not transmit data");

    println!("Success.");
}
