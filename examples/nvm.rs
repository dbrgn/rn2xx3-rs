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

    println!("Write 23 to 0x300");
    rn.nvm_set(0x300, 23).unwrap();
    println!("Write 42 to 0x3ff");
    rn.nvm_set(0x3ff, 42).unwrap();
    println!("Read 0x300 -> {}", rn.nvm_get(0x300).unwrap());
    println!("Read 0x3ff -> {}", rn.nvm_get(0x3ff).unwrap());

    println!("--");

    println!("Write 42 to 0x300");
    rn.nvm_set(0x300, 42).unwrap();
    println!("Write 23 to 0x3ff");
    rn.nvm_set(0x3ff, 23).unwrap();
    println!("Read 0x300 -> {}", rn.nvm_get(0x300).unwrap());
    println!("Read 0x3ff -> {}", rn.nvm_get(0x3ff).unwrap());
}
