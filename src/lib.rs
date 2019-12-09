//! # RN2xx3
//!
//! Driver for Microchip RN2483 / RN2903 LoRaWAN modules.
//!
//! ## Usage
//!
//! First, configure a serial port using a crate that implements the serial
//! traits from `embedded_hal`, for example
//! [serial](https://crates.io/crates/serial).
//!
//! ```no_run
//! use std::time::Duration;
//! use linux_embedded_hal::Serial;
//! use serial::{self, core::SerialPort};
//!
//! // Serial port settings
//! let settings = serial::PortSettings {
//!     baud_rate: serial::Baud57600,
//!     char_size: serial::Bits8,
//!     parity: serial::ParityNone,
//!     stop_bits: serial::Stop1,
//!     flow_control: serial::FlowNone,
//! };
//!
//! // Open serial port
//! let mut port = serial::open("/dev/ttyACM0").expect("Could not open serial port");
//! port.configure(&settings)
//!     .expect("Could not configure serial port");
//! port.set_timeout(Duration::from_secs(1))
//!     .expect("Could not set serial port timeout");
//! let serialport = Serial(port);
//! ```
//!
//! Then initialize the driver, either for the RN2483 or for the RN2903, on
//! your desired frequency:
//!
//! ```no_run
//! # use std::time::Duration;
//! # use linux_embedded_hal::Serial;
//! # use serial::{self, core::SerialPort};
//! # // Serial port settings
//! # let settings = serial::PortSettings {
//! #     baud_rate: serial::Baud57600,
//! #     char_size: serial::Bits8,
//! #     parity: serial::ParityNone,
//! #     stop_bits: serial::Stop1,
//! #     flow_control: serial::FlowNone,
//! # };
//! use rn2xx3;
//!
//! # let mut port = serial::open("/dev/ttyACM0").expect("Could not open serial port");
//! # port.configure(&settings).expect("Could not configure serial port");
//! # port.set_timeout(Duration::from_secs(1)).expect("Could not set serial port timeout");
//! # let serialport = Serial(port);
//! // RN2483 at 868 MHz
//! let rn = rn2xx3::rn2483_868(serialport);
//!
//! # let mut port = serial::open("/dev/ttyACM0").expect("Could not open serial port");
//! # port.configure(&settings).expect("Could not configure serial port");
//! # port.set_timeout(Duration::from_secs(1)).expect("Could not set serial port timeout");
//! # let serialport = Serial(port);
//! // RN2483 at 433 MHz
//! let rn = rn2xx3::rn2483_433(serialport);
//!
//! # let mut port = serial::open("/dev/ttyACM0").expect("Could not open serial port");
//! # port.configure(&settings).expect("Could not configure serial port");
//! # port.set_timeout(Duration::from_secs(1)).expect("Could not set serial port timeout");
//! # let serialport = Serial(port);
//! // RN2903 at 915 MHz
//! let rn = rn2xx3::rn2903_915(serialport);
//! ```
//!
//! Now you can read information from the module and join, e.g. via OTAA:
//!
//! ```no_run
//! # use std::time::Duration;
//! # use linux_embedded_hal::Serial;
//! # use serial::{self, core::SerialPort};
//! # // Serial port settings
//! # let settings = serial::PortSettings {
//! #     baud_rate: serial::Baud57600,
//! #     char_size: serial::Bits8,
//! #     parity: serial::ParityNone,
//! #     stop_bits: serial::Stop1,
//! #     flow_control: serial::FlowNone,
//! # };
//! # use rn2xx3;
//! # let mut port = serial::open("/dev/ttyACM0").expect("Could not open serial port");
//! # port.configure(&settings).expect("Could not configure serial port");
//! # port.set_timeout(Duration::from_secs(1)).expect("Could not set serial port timeout");
//! # let serialport = Serial(port);
//! # let mut rn = rn2xx3::rn2483_868(serialport);
//! use rn2xx3::{ConfirmationMode, JoinMode};
//!
//! // Reset module
//! println!("Resetting module...\n");
//! rn.reset().expect("Could not reset");
//!
//! // Show device info
//! println!("== Device info ==\n");
//! let hweui = rn.hweui().expect("Could not read hweui");
//! println!("     HW-EUI: {}", hweui);
//! let model = rn.model().expect("Could not read model");
//! println!("      Model: {:?}", model);
//! let version = rn.version().expect("Could not read version");
//! println!("    Version: {}", version);
//! let vdd = rn.vdd().expect("Could not read vdd");
//! println!("Vdd voltage: {} mV", vdd);
//!
//! // Set keys
//! println!("Setting keys...");
//! rn.set_app_eui_hex("0011223344556677").expect("Could not set app EUI");
//! rn.set_app_key_hex("0011223344556677889900aabbccddee").expect("Could not set app key");
//!
//! // Join
//! println!("Joining via OTAA...");
//! rn.join(JoinMode::Otaa).expect("Could not join");
//! println!("OK");
//!
//! // Send data
//! let fport = 1u8;
//! rn.transmit_slice(ConfirmationMode::Unconfirmed, fport, &[23, 42]).expect("Could not transmit data");
//! ```
//!
//! For more examples, refer to the `examples` directory in the source repository.
mod errors;
mod utils;

use core::marker::PhantomData;
use core::str::{from_utf8, FromStr};

use base16;
use doc_comment::doc_comment;
use embedded_hal::serial;
use nb::block;

use crate::errors::{Error, JoinError, TxError, RnResult};

const CR: u8 = 0x0d;
const LF: u8 = 0x0a;
const OK: [u8; 2] = [b'o', b'k'];

/// Marker trait implemented for all models / frequencies.
pub trait Frequency {}
/// Frequency type parameter for the RN2483 (433 MHz).
pub struct Freq433;
/// Frequency type parameter for the RN2483 (868 MHz).
pub struct Freq868;
/// Frequency type parameter for the RN2903 (915 MHz).
pub struct Freq915;
impl Frequency for Freq433 {}
impl Frequency for Freq868 {}
impl Frequency for Freq915 {}

/// The main driver instance.
pub struct Driver<F: Frequency, S> {
    frequency: PhantomData<F>,
    serial: S,
    read_buf: [u8; 64],
}

/// List of all supported RN module models.
#[derive(Debug, PartialEq, Eq)]
pub enum Model {
    RN2483,
    RN2903,
}

/// The join procedure.
#[derive(Debug, PartialEq, Eq)]
pub enum JoinMode {
    /// Over the air activation
    Otaa,
    /// Activation by personalization
    Abp,
}

/// Whether to send an uplink as confirmed or unconfirmed message.
#[derive(Debug, PartialEq, Eq)]
pub enum ConfirmationMode {
    /// Expect a confirmation from the gateway.
    Confirmed,
    /// No confirmation is expected.
    Unconfirmed,
}

#[derive(Debug, PartialEq)]
pub struct Downlink<'a> {
    port: u8,
    hexdata: &'a str,
}

/// Create a new driver instance for the RN2483 (433 MHz), wrapping the
/// specified serial port.
pub fn rn2483_433<S, E>(serial: S) -> Driver<Freq433, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    Driver {
        frequency: PhantomData,
        serial,
        read_buf: [0; 64],
    }
}

/// Create a new driver instance for the RN2483 (868 MHz), wrapping the
/// specified serial port.
pub fn rn2483_868<S, E>(serial: S) -> Driver<Freq868, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    Driver {
        frequency: PhantomData,
        serial,
        read_buf: [0; 64],
    }
}

/// Create a new driver instance for the RN2903 (915 MHz), wrapping the
/// specified serial port.
pub fn rn2903_915<S, E>(serial: S) -> Driver<Freq915, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    Driver {
        frequency: PhantomData,
        serial,
        read_buf: [0; 64],
    }
}

/// Basic commands.
impl<F, S, E> Driver<F, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    F: Frequency,
{
    /// Write a single byte to the serial port.
    fn write_byte(&mut self, byte: u8) -> RnResult<()> {
        block!(self.serial.write(byte)).map_err(|_| Error::SerialWrite)
    }

    fn write_crlf(&mut self) -> RnResult<()> {
        self.write_byte(CR)?;
        self.write_byte(LF)
    }

    /// Write all bytes from the buffer to the serial port.
    fn write_all(&mut self, buffer: &[u8]) -> RnResult<()> {
        for byte in buffer {
            self.write_byte(*byte)?;
        }
        Ok(())
    }

    /// Read a single byte from the serial port.
    fn read_byte(&mut self) -> RnResult<u8> {
        block!(self.serial.read()).map_err(|_| Error::SerialRead)
    }

    /// Read a CR/LF terminated line from the serial port.
    ///
    /// The string is returned without the line termination.
    fn read_line(&mut self) -> RnResult<&[u8]> {
        let buflen = self.read_buf.len();
        let mut i = 0;
        loop {
            match self.read_byte()? {
                LF if self.read_buf[i - 1] == CR => {
                    return Ok(&self.read_buf[0..(i - 1)]);
                }
                other => {
                    self.read_buf[i] = other;
                }
            }
            i += 1;
            if i >= buflen {
                return Err(Error::ReadBufferTooSmall);
            }
        }
    }

    /// Send a raw command to the module and return the response.
    pub fn send_raw_command(&mut self, command: &[&str]) -> RnResult<&[u8]> {
        for part in command {
            self.write_all(part.as_bytes())?;
        }
        self.write_crlf()?;
        self.read_line()
    }

    /// Send a raw command and decode the resulting bytes to a `&str`.
    pub fn send_raw_command_str(&mut self, command: &[&str]) -> RnResult<&str> {
        let bytes = self.send_raw_command(command)?;
        Ok(from_utf8(bytes)?)
    }

    /// Send a raw command that should be confirmed with 'OK'. If the response
    /// is not 'OK', return `Error::CommandFailed`.
    fn send_raw_command_ok(&mut self, command: &[&str]) -> RnResult<()> {
        let response = self.send_raw_command(command)?;
        if response == &OK {
            Ok(())
        } else {
            Err(Error::CommandFailed)
        }
    }
}

/// System commands.
impl<F, S, E> Driver<F, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    F: Frequency,
{
    /// Reset and restart the RN module. Return the version string.
    pub fn reset(&mut self) -> RnResult<&str> {
        self.send_raw_command_str(&["sys reset"])
    }

    /// Reset the module's configuration data and user EEPROM to factory
    /// default values and restart the module.
    ///
    /// All configuration parameters will be restored to factory default
    /// values. Return the version string.
    pub fn factory_reset(&mut self) -> RnResult<&str> {
        self.send_raw_command_str(&["sys factoryRESET"])
    }

    ///// Delete the current RN2483 module application firmware and prepare it
    ///// for firmware upgrade. The module bootloader is then ready to receive
    ///// new firmware.
    /////
    ///// This command is not unsafe in the sense of memory unsafety, but it can
    ///// be dangerous because it removes the firmware.
    //pub unsafe fn erase_fw(&mut self) -> RnResult<()> {
    //    self.send_raw_command(&["sys eraseFW"])?;
    //    TODO: Does this return anything?
    //    Ok(())
    //}

    /// Return the preprogrammed EUI node address as uppercase hex string.
    pub fn hweui(&mut self) -> RnResult<&str> {
        self.send_raw_command_str(&["sys get hweui"])
    }

    /// Return the version string.
    pub fn version(&mut self) -> RnResult<&str> {
        self.send_raw_command_str(&["sys get ver"])
    }

    /// Return the model of the module.
    pub fn model(&mut self) -> RnResult<Model> {
        let version = self.version()?;
        match &version[0..6] {
            "RN2483" => Ok(Model::RN2483),
            "RN2903" => Ok(Model::RN2903),
            _ => Err(Error::ParsingError),
        }
    }

    /// Measure and return the Vdd voltage in millivolts.
    pub fn vdd(&mut self) -> RnResult<u16> {
        let vdd = self.send_raw_command_str(&["sys get vdd"])?;
        vdd.parse().map_err(|_| Error::ParsingError)
    }

    /// Set the NVM byte at `addr` to the specified value.
    ///
    /// The address must be between 0x300 and 0x3ff, otherwise
    /// `Error::BadParameter` is returned.
    pub fn nvm_set(&mut self, addr: u16, byte: u8) -> RnResult<()> {
        if addr < 0x300 || addr > 0x3ff {
            return Err(Error::BadParameter);
        }
        let hex_addr = format!("{:x}", addr);
        let hex_byte_bytes = base16::encode_byte_l(byte);
        let hex_byte = from_utf8(&hex_byte_bytes).unwrap();
        let args = ["sys set nvm ", &hex_addr, " ", &hex_byte];
        self.send_raw_command_ok(&args)
    }

    /// Get the NVM byte at `addr`.
    ///
    /// The address must be between 0x300 and 0x3ff, otherwise
    /// `Error::BadParameter` is returned.
    pub fn nvm_get(&mut self, addr: u16) -> RnResult<u8> {
        if addr < 0x300 || addr > 0x3ff {
            return Err(Error::BadParameter);
        }
        let hex_addr = format!("{:x}", addr);
        let response = self.send_raw_command(&["sys get nvm ", &hex_addr])?;
        if response.len() != 2 {
            return Err(Error::ParsingError);
        }
        let mut buf = [0; 1];
        base16::decode_slice(response, &mut buf).map_err(|_| Error::ParsingError)?;
        Ok(buf[0])
    }
}

macro_rules! hex_setter_getter {
    (
        $field:expr, $bytes:expr,
        $descr:expr,
        $set_hex:ident, $set_slice:ident,
        $get_hex:ident, $get_slice:ident,
        $(,)?
    ) => {
        doc_comment! {
            concat!(
                "Set ",
                $descr,
                ".",
                "\n\nThe parameter must be a ", stringify!($bytes), "-byte hex string, ",
                "otherwise `Error::BadParameter` will be returned.",
            ),
            pub fn $set_hex(&mut self, val: &str) -> RnResult<()> {
                if val.len() != $bytes * 2 {
                    return Err(Error::BadParameter);
                }
                self.send_raw_command_ok(&[concat!("mac set ", $field, " "), val])
            }
        }

        doc_comment! {
            concat!(
                "Set ",
                $descr,
                ".",
                "\n\nThe parameter must be a ", stringify!($bytes), "-byte ",
                "big endian byte slice, otherwise `Error::BadParameter` will be returned.",
            ),
            pub fn $set_slice(&mut self, val: &[u8]) -> RnResult<()> {
                if val.len() != $bytes {
                    return Err(Error::BadParameter);
                }
                let mut buf = [0; $bytes * 2];
                base16::encode_config_slice(val, base16::EncodeLower, &mut buf);
                self.$set_hex(from_utf8(&buf)?)
            }
        }

        doc_comment! {
            concat!("Get ", $descr, " as hex str."),
            pub fn $get_hex(&mut self) -> RnResult<&str> {
                self.send_raw_command_str(&[concat!("mac get ", $field)])
            }
        }

        doc_comment! {
            concat!("Get ", $descr, " bytes."),
            pub fn $get_slice(&mut self) -> RnResult<[u8; $bytes]> {
                let hex = self.$get_hex()?;
                let mut buf = [0; $bytes];
                base16::decode_slice(hex, &mut buf).map_err(|_| Error::ParsingError)?;
                Ok(buf)
            }
        }
    }
}

/// MAC commands.
impl<F, S, E> Driver<F, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    F: Frequency,
{
    /// Save MAC configuration parameters.
    ///
    /// This command will save LoRaWAN Class A protocol configuration
    /// parameters to the user EEPROM. When the next [`sys
    /// reset`](#method.reset) command is issued, the LoRaWAN Class A protocol
    /// configuration will be initialized with the last saved parameters.
    ///
    /// The LoRaWAN Class A protocol configuration savable parameters are:
    /// `band`, `deveui`, `appeui`, `appkey`, `nwkskey`, `appskey`, `devaddr`,
    /// `ch freq`, `ch dcycle`, `ch drrange`, `ch status`.
    pub fn save_config(&mut self) -> RnResult<()> {
        self.send_raw_command_ok(&["mac save"])
    }

    hex_setter_getter!(
        "devaddr", 4,
        "the unique network device address",
        set_dev_addr_hex,
        set_dev_addr_slice,
        get_dev_addr_hex,
        get_dev_addr_slice,
    );

    hex_setter_getter!(
        "deveui", 8,
        "the globally unique device identifier",
        set_dev_eui_hex,
        set_dev_eui_slice,
        get_dev_eui_hex,
        get_dev_eui_slice,
    );

    hex_setter_getter!(
        "appeui", 8,
        "the globally unique application identifier",
        set_app_eui_hex,
        set_app_eui_slice,
        get_app_eui_hex,
        get_app_eui_slice,
    );

    hex_setter_getter!(
        "nwkskey", 16,
        "the network session key",
        set_network_session_key_hex,
        set_network_session_key_slice,
        get_network_session_key_hex,
        get_network_session_key_slice,
    );

    hex_setter_getter!(
        "appskey", 16,
        "the application session key",
        set_app_session_key_hex,
        set_app_session_key_slice,
        get_app_session_key_hex,
        get_app_session_key_slice,
    );

    hex_setter_getter!(
        "appkey", 16,
        "the application key",
        set_app_key_hex,
        set_app_key_slice,
        get_app_key_hex,
        get_app_key_slice,
    );

    /// Join the network.
    pub fn join(&mut self, mode: JoinMode) -> Result<(), JoinError> {
        let mode_str = match mode {
            JoinMode::Otaa => "otaa",
            JoinMode::Abp => "abp",
        };

        // First response is whether the join procedure was initialized properly.
        match self.send_raw_command_str(&["mac join ", mode_str])? {
            "ok" => {},
            "invalid_param" => return Err(JoinError::BadParameter),
            "keys_not_init" => return Err(JoinError::KeysNotInit),
            "no_free_ch" => return Err(JoinError::NoFreeChannel),
            "silent" => return Err(JoinError::Silent),
            "busy" => return Err(JoinError::Busy),
            "mac_paused" => return Err(JoinError::MacPaused),
            "denied" => return Err(JoinError::JoinUnsuccessful),
            _ => return Err(JoinError::UnknownResponse),
        };

        // Second response indicates whether the join procedure succeeded.
        match self.read_line()? {
            b"denied" => Err(JoinError::JoinUnsuccessful),
            b"accepted" => Ok(()),
            _ => Err(JoinError::UnknownResponse),
        }
    }

    /// Send a hex uplink on the specified port.
    ///
    /// If a downlink is received, it is returned.
    pub fn transmit_hex(
        &mut self,
        mode: ConfirmationMode,
        port: u8,
        data: &str,
    ) -> Result<Option<Downlink>, TxError> {
        // Validate and parse arguments
        if data.len() % 2 != 0 {
            return Err(TxError::BadParameter);
        }
        utils::validate_port(port, TxError::BadParameter)?;
        let mode_str = match mode {
            ConfirmationMode::Confirmed => "cnf",
            ConfirmationMode::Unconfirmed => "uncnf",
        };
        let mut buf = [0; 3];
        let port_str = utils::u8_to_str(port, &mut buf)?;

        // First response is whether the uplink transmission could be initialized.
        match self.send_raw_command(&["mac tx ", mode_str, " ", port_str, " ", data])? {
            b"ok" => {},
            b"invalid_param" => return Err(TxError::BadParameter),
            b"not_joined" => return Err(TxError::NotJoined),
            b"no_free_ch" => return Err(TxError::NoFreeChannel),
            b"silent" => return Err(TxError::Silent),
            b"frame_counter_err_rejoin_needed" => return Err(TxError::FrameCounterRollover),
            b"busy" => return Err(TxError::Busy),
            b"mac_paused" => return Err(TxError::MacPaused),
            b"invalid_data_len" => return Err(TxError::InvalidDataLenth),
            _ => return Err(TxError::UnknownResponse),
        };

        // The second response could contain an error or a downlink.
        match self.read_line()? {
            b"mac_tx_ok" => Ok(None),
            b"mac_err" => Err(TxError::TxUnsuccessful),
            b"invalid_data_len" => return Err(TxError::InvalidDataLenth),
            val if val.starts_with(b"mac_rx ") => {
                let mut parts = from_utf8(val)?.split_ascii_whitespace();

                // Get port
                let _ = parts.next().ok_or(TxError::Other(Error::ParsingError))?;
                let port_str = parts.next().ok_or(TxError::Other(Error::ParsingError))?;
                let port = u8::from_str(&port_str)
                    .map_err(|_| TxError::Other(Error::ParsingError))?;
                utils::validate_port(port, TxError::Other(Error::ParsingError))?;

                // Get data
                let hexdata = parts.next().ok_or(TxError::Other(Error::ParsingError))?;
                if hexdata.len() % 2 != 0 {
                    return Err(TxError::Other(Error::ParsingError));
                }

                Ok(Some(Downlink { port, hexdata }))
            },
            _ => Err(TxError::UnknownResponse),
        }
    }

    /// Send an uplink on the specified port.
    ///
    /// If a downlink is received, it is returned.
    pub fn transmit_slice(
        &mut self,
        mode: ConfirmationMode,
        port: u8,
        data: &[u8],
    ) -> Result<Option<Downlink>, TxError> {
        let mut buf = [0; 256];
        let bytes = base16::encode_config_slice(data, base16::EncodeLower, &mut buf);
        self.transmit_hex(mode, port, from_utf8(&buf[0..bytes])?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use embedded_hal_mock::serial::{Mock as SerialMock, Transaction};

    const VERSION48: &str = "RN2483 1.0.3 Mar 22 2017 06:00:42";
    const VERSION90: &str = "RN2903 1.0.3 Mar 22 2017 06:00:42";
    const CRLF: &str = "\r\n";

    #[test]
    fn version() {
        let expectations = [
            Transaction::write_many(b"sys get ver\r\n"),
            Transaction::read_many(VERSION48.as_bytes()),
            Transaction::read_many(CRLF.as_bytes()),
        ];
        let mut mock = SerialMock::new(&expectations);
        let mut rn = rn2483_868(mock.clone());
        assert_eq!(rn.version().unwrap(), VERSION48);
        mock.done();
    }

    #[test]
    fn model_rn2483() {
        let expectations = [
            Transaction::write_many(b"sys get ver\r\n"),
            Transaction::read_many(VERSION48.as_bytes()),
            Transaction::read_many(CRLF.as_bytes()),
        ];
        let mut mock = SerialMock::new(&expectations);
        let mut rn = rn2483_868(mock.clone());
        assert_eq!(rn.model().unwrap(), Model::RN2483);
        mock.done();
    }

    #[test]
    fn model_rn2903() {
        let expectations = [
            Transaction::write_many(b"sys get ver\r\n"),
            Transaction::read_many(VERSION90.as_bytes()),
            Transaction::read_many(CRLF.as_bytes()),
        ];
        let mut mock = SerialMock::new(&expectations);
        let mut rn = rn2483_868(mock.clone());
        assert_eq!(rn.model().unwrap(), Model::RN2903);
        mock.done();
    }

    #[test]
    fn nvm_set() {
        let expectations = [
            Transaction::write_many(b"sys set nvm 3ab 2a\r\n"),
            Transaction::read_many(b"ok\r\n"),
        ];
        let mut mock = SerialMock::new(&expectations);
        let mut rn = rn2483_868(mock.clone());
        rn.nvm_set(0x3ab, 42).unwrap();
        mock.done();
    }

    #[test]
    fn nvm_get() {
        let expectations = [
            Transaction::write_many(b"sys get nvm 300\r\n"),
            Transaction::read_many(b"ff\r\n"),
        ];
        let mut mock = SerialMock::new(&expectations);
        let mut rn = rn2483_868(mock.clone());
        assert_eq!(rn.nvm_get(0x300).unwrap(), 0xff);
        mock.done();
    }

    /// Validate length of value passed to generated methods.
    /// Ensure that nothing is read/written to/from the serial device.
    #[test]
    fn set_dev_addr_bad_length() {
        let expectations = [];
        let mut mock = SerialMock::new(&expectations);
        let mut rn = rn2483_868(mock.clone());
        assert_eq!(rn.set_dev_addr_hex("010203f"), Err(Error::BadParameter));
        assert_eq!(rn.set_dev_addr_hex("010203fff"), Err(Error::BadParameter));
        assert_eq!(rn.set_dev_eui_hex("0004a30b001a55e"), Err(Error::BadParameter));
        assert_eq!(rn.set_dev_eui_hex("0004a30b001a55edx"), Err(Error::BadParameter));
        mock.done();
    }

    fn _set_dev_addr() -> (SerialMock<u8>, Driver<Freq868, SerialMock<u8>>) {
        let expectations = [
            Transaction::write_many(b"mac set devaddr 010203ff\r\n"),
            Transaction::read_many(b"ok\r\n"),
        ];
        let mock = SerialMock::new(&expectations);
        let rn = rn2483_868(mock.clone());
        (mock, rn)
    }

    #[test]
    fn set_dev_addr_hex() {
        let (mut mock, mut rn) = _set_dev_addr();
        assert!(rn.set_dev_addr_hex("010203ff").is_ok());
        mock.done();
    }

    #[test]
    fn set_dev_addr_slice() {
        let (mut mock, mut rn) = _set_dev_addr();
        assert!(rn.set_dev_addr_slice(&[0x01, 0x02, 0x03, 0xff]).is_ok());
        mock.done();
    }

    fn _set_dev_eui() -> (SerialMock<u8>, Driver<Freq868, SerialMock<u8>>) {
        let expectations = [
            Transaction::write_many(b"mac set deveui 0004a30b001a55ed\r\n".as_ref()),
            Transaction::read_many(b"ok\r\n"),
        ];
        let mock = SerialMock::new(&expectations);
        let rn = rn2483_868(mock.clone());
        (mock, rn)
    }

    #[test]
    fn set_dev_eui_hex() {
        let (mut mock, mut rn) = _set_dev_eui();
        assert!(rn.set_dev_eui_hex("0004a30b001a55ed").is_ok());
        mock.done();
    }

    #[test]
    fn set_dev_eui_slice() {
        let (mut mock, mut rn) = _set_dev_eui();
        assert!(rn.set_dev_eui_slice(&[0x00, 0x04, 0xa3, 0x0b, 0x00, 0x1a, 0x55, 0xed]).is_ok());
        mock.done();
    }

    mod join {
        use super::*;

        #[test]
        fn otaa_ok() {
            let expectations = [
                Transaction::write_many(b"mac join otaa\r\n"),
                Transaction::read_many(b"ok\r\naccepted\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.join(JoinMode::Otaa), Ok(()));
            mock.done();
        }

        #[test]
        fn abp_ok() {
            let expectations = [
                Transaction::write_many(b"mac join abp\r\n"),
                Transaction::read_many(b"ok\r\naccepted\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.join(JoinMode::Abp), Ok(()));
            mock.done();
        }

        #[test]
        fn otaa_denied() {
            let expectations = [
                Transaction::write_many(b"mac join otaa\r\n"),
                Transaction::read_many(b"ok\r\ndenied\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.join(JoinMode::Otaa), Err(JoinError::JoinUnsuccessful));
            mock.done();
        }

        #[test]
        fn otaa_unknown_response_1() {
            let expectations = [
                Transaction::write_many(b"mac join otaa\r\n"),
                Transaction::read_many(b"xyz\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.join(JoinMode::Otaa), Err(JoinError::UnknownResponse));
            mock.done();
        }

        #[test]
        fn otaa_unknown_response_2() {
            let expectations = [
                Transaction::write_many(b"mac join otaa\r\n"),
                Transaction::read_many(b"ok\r\nxyz\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.join(JoinMode::Otaa), Err(JoinError::UnknownResponse));
            mock.done();
        }

        #[test]
        fn otaa_no_free_ch() {
            let expectations = [
                Transaction::write_many(b"mac join otaa\r\n"),
                Transaction::read_many(b"no_free_ch\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.join(JoinMode::Otaa), Err(JoinError::NoFreeChannel));
            mock.done();
        }
    }

    mod transmit {
        use super::*;

        #[test]
        fn transmit_hex_uncnf_no_downlink() {
            let expectations = [
                Transaction::write_many(b"mac tx uncnf 42 23ff\r\n"),
                Transaction::read_many(b"ok\r\nmac_tx_ok\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.transmit_hex(ConfirmationMode::Unconfirmed, 42, "23ff"), Ok(None));
            mock.done();
        }

        #[test]
        fn transmit_hex_cnf_no_downlink() {
            let expectations = [
                Transaction::write_many(b"mac tx cnf 42 23ff\r\n"),
                Transaction::read_many(b"ok\r\nmac_tx_ok\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.transmit_hex(ConfirmationMode::Confirmed, 42, "23ff"), Ok(None));
            mock.done();
        }

        #[test]
        fn transmit_hex_uncnf_downlink() {
            let expectations = [
                Transaction::write_many(b"mac tx uncnf 42 23ff\r\n"),
                Transaction::read_many(b"ok\r\nmac_rx 101 000102feff\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(
                rn.transmit_hex(ConfirmationMode::Unconfirmed, 42, "23ff"),
                Ok(Some(Downlink {
                    port: 101,
                    hexdata: "000102feff",
                }))
            );
            mock.done();
        }

        #[test]
        fn transmit_slice_uncnf_no_downlink() {
            let expectations = [
                Transaction::write_many(b"mac tx uncnf 42 23ff\r\n"),
                Transaction::read_many(b"ok\r\nmac_tx_ok\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(
                rn.transmit_slice(ConfirmationMode::Unconfirmed, 42, &[0x23, 0xff]),
                Ok(None),
            );
            mock.done();
        }
    }
}
