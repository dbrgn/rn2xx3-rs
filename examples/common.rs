use std::time::Duration;

use linux_embedded_hal::Serial;
use rn2xx3::{Driver, Rn2483, rn2483};
use serial::{self, core::SerialPort};

pub fn init_rn(dev: &str) -> Driver<Rn2483, Serial> {
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
    rn2483(Serial(port))
}

#[allow(dead_code)]
pub fn main() {
    // Not a real example!
}
