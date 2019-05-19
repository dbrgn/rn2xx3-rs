use std::env;
use std::time::Duration;

use linux_embedded_hal::serial_impl::Serial;
use serial::{self, core::SerialPort};

use rn2xx3::Rn2483;

fn main() {
    println!("Hello, RN2483!");

    // Parse args
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <path-to-serial>", args[0]);
        println!("Example: {} /dev/ttyUSB0", args[0]);
        std::process::exit(1);
    }
    let dev = &args[1];

    // Serial port settings
    let settings = serial::PortSettings {
        baud_rate: serial::Baud57600,
        char_size: serial::Bits8,
        parity: serial::ParityNone,
        stop_bits: serial::Stop1,
        flow_control: serial::FlowNone,
    };

    // Open serial port
    let mut port = serial::open(dev).expect("Could not open serial port");
    port.configure(&settings)
        .expect("Could not configure serial port");
    port.set_timeout(Duration::from_secs(1))
        .expect("Could not set serial port timeout");

    // Initialize driver
    let mut rn = Rn2483::new(Serial(port));
    println!("hweui: {}", rn.hweui().expect("Could not read hweui"));
}
