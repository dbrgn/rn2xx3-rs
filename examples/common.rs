use std::time::Duration;

use linux_embedded_hal::serial_impl::Serial;
use rn2xx3::Rn2xx3;
use serial::{self, core::SerialPort};

pub fn init_rn(dev: &str) -> Rn2xx3<Serial> {
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
    Rn2xx3::new(Serial(port))
}

#[allow(dead_code)]
pub fn main() {
    // Not a real example!
}
