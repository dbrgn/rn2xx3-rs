mod common;

use std::env;
use std::time::Duration;

use env_logger;
use rn2xx3::errors::Error;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <path-to-serial>", args[0]);
        println!("Example: {} /dev/ttyUSB0", args[0]);
        std::process::exit(1);
    }
    let mut rn = common::init_rn(&args[1]);

    println!("Resetting module...\n");
    rn.reset().expect("Could not reset");

    println!("Reading hweui...");
    let hweui = rn.hweui().expect("Could not read hweui");
    println!("> HW-EUI: {}", hweui);

    println!("\nPutting module to sleep for 2 seconds...");
    rn.sleep(Duration::from_secs(2))
        .expect("Could not enable sleep mode");
    println!("> Done");

    println!("\nReading hweui...");
    match rn.hweui().unwrap_err() {
        Error::SleepMode => println!("> Failed: Module still in sleep mode"),
        other => println!("> Failed: Unexpected error: {:?}", other),
    }

    println!("Waiting for wakeup...");
    rn.wait_for_wakeup(false)
        .expect("Waiting for wakeup failed");
    println!("> Ok");

    println!("Reading hweui...");
    let hweui = rn.hweui().expect("Could not read hweui");
    println!("> HW-EUI: {}", hweui);
}
