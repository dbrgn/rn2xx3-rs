//! # RN2xx3
//!
//! A `no_std` / `embedded_hal` compatible Rust driver for the RN2483 and
//! the RN2903 LoRaWAN modules. The library works without any dynamic allocations.
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
//! After initializing, it's a good idea to clear the serial buffers and ensure
//! a "known good state".
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
//! rn.ensure_known_state().expect("Error while preparing device");
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
//!
//! ## Logging
//!
//! If you are running the driver from a platform that has access to `std`, you
//! can also enable the optional `logging` feature to be able to see incoming
//! and outgoing commands:
//!
//! ```text
//! $ export RUST_LOG=debug
//! $ cargo run --features logging --example join_otaa ...
//! Resetting module...
//! [2020-03-03T20:41:42Z DEBUG rn2xx3] Sending command: "sys reset"
//! [2020-03-03T20:41:42Z DEBUG rn2xx3] Received response: "RN2483 1.0.3 Mar 22 2017 06:00:42"
//! ...
//! ```

#![cfg_attr(not(test), no_std)]

pub mod errors;
mod utils;

use core::convert::TryFrom;
use core::marker::PhantomData;
use core::str::{from_utf8, FromStr};
use core::time::Duration;

use doc_comment::doc_comment;
use embedded_hal::serial;
use nb::block;
use numtoa::NumToA;

#[cfg(feature = "logging")]
use core::fmt;
#[cfg(feature = "logging")]
use log;

use crate::errors::{Error, JoinError, RnResult, TxError};

const CR: u8 = 0x0d;
const LF: u8 = 0x0a;

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

#[cfg(feature = "logging")]
struct LoggableStrSlice<'o, 'i>(&'o [&'i str]);

#[cfg(feature = "logging")]
impl fmt::Display for LoggableStrSlice<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for part in self.0 {
            write!(f, "{}", part)?;
        }
        Ok(())
    }
}

/// The main driver instance.
pub struct Driver<F: Frequency, S> {
    /// Marker type with the module frequency.
    frequency: PhantomData<F>,

    /// Serial port.
    serial: S,

    /// Read buffer.
    read_buf: [u8; 64],

    /// This flag is set when entering sleep mode. As long as it is set,
    /// sending any command will be prevented.
    sleep: bool,
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

/// The data rates valid in Europe and China.
///
/// Frequencies:
///
/// - EU 863–870 MHz (LoRaWAN Specification (2015), Page 35, Table 14)
/// - CN 779–787 MHz (LoRaWAN Specification (2015), Page 44, Table 25)
/// - EU 433 MHz (LoRaWAN Specification (2015), Page 48, Table 31)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DataRateEuCn {
    /// Data Rate 0: SF 12 BW 125 (250 bit/s)
    Sf12Bw125,
    /// Data Rate 1: SF 11 BW 125 (440 bit/s)
    Sf11Bw125,
    /// Data Rate 2: SF 10 BW 125 (980 bit/s)
    Sf10Bw125,
    /// Data Rate 3: SF 9 BW 125 (1760 bit/s)
    Sf9Bw125,
    /// Data Rate 4: SF 8 BW 125 (3125 bit/s)
    Sf8Bw125,
    /// Data Rate 5: SF 7 BW 125 (5470 bit/s)
    Sf7Bw125,
    /// Data Rate 6: SF 7 BW 250 (11000 bit/s)
    Sf7Bw250,
}

impl From<DataRateEuCn> for &str {
    fn from(dr: DataRateEuCn) -> Self {
        match dr {
            DataRateEuCn::Sf12Bw125 => "0",
            DataRateEuCn::Sf11Bw125 => "1",
            DataRateEuCn::Sf10Bw125 => "2",
            DataRateEuCn::Sf9Bw125 => "3",
            DataRateEuCn::Sf8Bw125 => "4",
            DataRateEuCn::Sf7Bw125 => "5",
            DataRateEuCn::Sf7Bw250 => "6",
        }
    }
}

impl TryFrom<&str> for DataRateEuCn {
    type Error = ();
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "0" => Ok(DataRateEuCn::Sf12Bw125),
            "1" => Ok(DataRateEuCn::Sf11Bw125),
            "2" => Ok(DataRateEuCn::Sf10Bw125),
            "3" => Ok(DataRateEuCn::Sf9Bw125),
            "4" => Ok(DataRateEuCn::Sf8Bw125),
            "5" => Ok(DataRateEuCn::Sf7Bw125),
            "6" => Ok(DataRateEuCn::Sf7Bw250),
            _ => Err(()),
        }
    }
}

/// The data rates valid in the USA.
///
/// Frequencies:
///
/// - US 902–928 MHz (LoRaWAN Specification (2015), Page 40, Table 18)
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DataRateUs {
    /// Data Rate 0: SF 10 BW 125 (980 bit/s)
    Sf10Bw125,
    /// Data Rate 1: SF 9 BW 125 (1760 bit/s)
    Sf9Bw125,
    /// Data Rate 2: SF 8 BW 125 (3125 bit/s)
    Sf8Bw125,
    /// Data Rate 3: SF 7 BW 125 (5470 bit/s)
    Sf7Bw125,
    /// Data Rate 4: SF 8 BW 500 (12500 bit/s)
    Sf8Bw500,
}

impl From<DataRateUs> for &str {
    fn from(dr: DataRateUs) -> Self {
        match dr {
            DataRateUs::Sf10Bw125 => "0",
            DataRateUs::Sf9Bw125 => "1",
            DataRateUs::Sf8Bw125 => "2",
            DataRateUs::Sf7Bw125 => "3",
            DataRateUs::Sf8Bw500 => "4",
        }
    }
}

impl TryFrom<&str> for DataRateUs {
    type Error = ();
    fn try_from(val: &str) -> Result<Self, Self::Error> {
        match val {
            "0" => Ok(DataRateUs::Sf10Bw125),
            "1" => Ok(DataRateUs::Sf9Bw125),
            "2" => Ok(DataRateUs::Sf8Bw125),
            "3" => Ok(DataRateUs::Sf7Bw125),
            "4" => Ok(DataRateUs::Sf8Bw500),
            _ => Err(()),
        }
    }
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
        sleep: false,
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
        sleep: false,
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
        sleep: false,
    }
}

/// Basic commands.
impl<F, S, E> Driver<F, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    F: Frequency,
{
    /// Write a single byte to the serial port.
    ///
    /// **Note:** For performance reasons, the `sleep` flag is not being
    /// checked here. Make sure not to call this method while in sleep mode.
    fn write_byte(&mut self, byte: u8) -> RnResult<(), E> {
        block!(self.serial.write(byte)).map_err(Error::SerialWrite)
    }

    /// Ensure that the device is not currently in sleep mode.
    ///
    /// Returns `Error::SleepMode` if `self.sleep` is set.
    fn ensure_not_in_sleep_mode(&self) -> RnResult<(), E> {
        if self.sleep {
            Err(Error::SleepMode)
        } else {
            Ok(())
        }
    }

    /// Write CR+LF bytes.
    fn write_crlf(&mut self) -> RnResult<(), E> {
        self.ensure_not_in_sleep_mode()?;
        self.write_byte(CR)?;
        self.write_byte(LF)
    }

    /// Write all bytes from the buffer to the serial port.
    fn write_all(&mut self, buffer: &[u8]) -> RnResult<(), E> {
        self.ensure_not_in_sleep_mode()?;
        for byte in buffer {
            self.write_byte(*byte)?;
        }
        Ok(())
    }

    /// Read a single byte from the serial port.
    fn read_byte(&mut self) -> RnResult<u8, E> {
        block!(self.serial.read()).map_err(Error::SerialRead)
    }

    /// Read a CR/LF terminated line from the serial port.
    ///
    /// The string is returned without the line termination.
    pub fn read_line(&mut self) -> RnResult<&[u8], E> {
        let buflen = self.read_buf.len();
        let mut i = 0;
        loop {
            match self.read_byte()? {
                LF if self.read_buf[i - 1] == CR => {
                    #[cfg(feature = "logging")]
                    log::debug!(
                        "Received response: {:?}",
                        from_utf8(&self.read_buf[0..(i - 1)]).unwrap_or("\"[invalid-utf8]\"")
                    );
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

    /// Send a raw command to the module and do not wait for the response.
    ///
    /// **Note:** If you use this for a command that returns a response (e.g.
    /// `sleep`), you will have to manually read the response using the
    /// `read_line()` method.
    pub fn send_raw_command_nowait(&mut self, command: &[&str]) -> RnResult<(), E> {
        #[cfg(feature = "logging")]
        log::debug!("Sending command: \"{}\"", LoggableStrSlice(command));
        for part in command {
            self.write_all(part.as_bytes())?;
        }
        self.write_crlf()?;
        Ok(())
    }

    /// Send a raw command to the module and return the response.
    pub fn send_raw_command(&mut self, command: &[&str]) -> RnResult<&[u8], E> {
        self.send_raw_command_nowait(command)?;
        self.read_line()
    }

    /// Send a raw command and decode the resulting bytes to a `&str`.
    pub fn send_raw_command_str(&mut self, command: &[&str]) -> RnResult<&str, E> {
        let bytes = self.send_raw_command(command)?;
        Ok(from_utf8(bytes)?)
    }

    /// Send a raw command that should be confirmed with 'OK'. If the response
    /// is not 'OK', return `Error::CommandFailed`.
    fn send_raw_command_ok(&mut self, command: &[&str]) -> RnResult<(), E> {
        let response = self.send_raw_command(command)?;
        if response == b"ok" {
            Ok(())
        } else {
            Err(Error::CommandFailed)
        }
    }

    /// Clear the module serial buffers and ensure a known good state.
    ///
    /// ## Implementation details
    ///
    /// This is done by first reading and discarding all available bytes from
    /// the serial port.
    ///
    /// Afterwards, to ensure that there's no valid command in the input
    /// buffer, the letter 'z' is sent, followed by a newline. There is no
    /// valid command that ends with 'z' and it's not a valid hex character, so
    /// the module should return `invalid_param`. If it doesn't, the same
    /// procedure is repeated 2 more times until giving up.
    ///
    /// Unexpected errors while reading or writing are propagated to the
    /// caller.
    pub fn ensure_known_state(&mut self) -> RnResult<(), E> {
        // First, clear the input buffer
        loop {
            match self.serial.read() {
                Ok(_) => {
                    // A byte was returned, continue reading
                    #[cfg(feature = "logging")]
                    log::debug!("Clearing input buffer: Discarded 1 byte");
                }
                Err(nb::Error::WouldBlock) => break,
                Err(nb::Error::Other(e)) => return Err(Error::SerialRead(e)),
            }
        }
        #[cfg(feature = "logging")]
        log::debug!("Input buffer is clear");

        // Max 3 attempts
        for _ in 0..3 {
            #[cfg(feature = "logging")]
            log::debug!("Check whether module is in a known state, expecting \"invalid_param\"");

            // To ensure that there's no valid command in the input buffer, write
            // the letter 'z' followed by CRLF.
            self.write_byte(b'z')?;
            self.write_crlf()?;

            // Read the response, it should be "invalid_param".
            match self.read_line()? {
                b"invalid_param" => return Ok(()),
                _other => {
                    #[cfg(feature = "logging")]
                    log::debug!("Error: Module returned \"{:?}\"", _other);
                }
            }
        }

        // Should not happen™
        Err(Error::InvalidState)
    }
}

/// System commands.
impl<F, S, E> Driver<F, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    F: Frequency,
{
    /// Destroy this driver instance, return the wrapped serial device.
    pub fn free(self) -> S {
        self.serial
    }

    /// Reset and restart the RN module. Return the version string.
    pub fn reset(&mut self) -> RnResult<&str, E> {
        self.send_raw_command_str(&["sys reset"])
    }

    /// Reset the module's configuration data and user EEPROM to factory
    /// default values and restart the module.
    ///
    /// All configuration parameters will be restored to factory default
    /// values. Return the version string.
    pub fn factory_reset(&mut self) -> RnResult<&str, E> {
        self.send_raw_command_str(&["sys factoryRESET"])
    }

    ///// Delete the current RN2483 module application firmware and ensure_known_state it
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
    pub fn hweui(&mut self) -> RnResult<&str, E> {
        self.send_raw_command_str(&["sys get hweui"])
    }

    /// Return the version string.
    pub fn version(&mut self) -> RnResult<&str, E> {
        self.send_raw_command_str(&["sys get ver"])
    }

    /// Return the model of the module.
    pub fn model(&mut self) -> RnResult<Model, E> {
        let version = self.version()?;
        match &version[0..6] {
            "RN2483" => Ok(Model::RN2483),
            "RN2903" => Ok(Model::RN2903),
            _ => Err(Error::ParsingError),
        }
    }

    /// Measure and return the Vdd voltage in millivolts.
    pub fn vdd(&mut self) -> RnResult<u16, E> {
        let vdd = self.send_raw_command_str(&["sys get vdd"])?;
        vdd.parse().map_err(|_| Error::ParsingError)
    }

    /// Set the NVM byte at `addr` to the specified value.
    ///
    /// The address must be between 0x300 and 0x3ff, otherwise
    /// `Error::BadParameter` is returned.
    pub fn nvm_set(&mut self, addr: u16, byte: u8) -> RnResult<(), E> {
        if addr < 0x300 || addr > 0x3ff {
            return Err(Error::BadParameter);
        }

        let mut hex_addr_buf = [0; 4];
        let addr_buf_byte_count = base16::encode_config_slice(
            &addr.to_be_bytes(),
            base16::EncodeLower,
            &mut hex_addr_buf,
        );
        let hex_addr = from_utf8(&hex_addr_buf[..addr_buf_byte_count]).unwrap();

        let hex_byte_bytes = base16::encode_byte_l(byte);
        let hex_byte = from_utf8(&hex_byte_bytes).unwrap();

        let args = ["sys set nvm ", utils::ltrim_hex(&hex_addr), " ", &hex_byte];
        self.send_raw_command_ok(&args)
    }

    /// Get the NVM byte at `addr`.
    ///
    /// The address must be between 0x300 and 0x3ff, otherwise
    /// `Error::BadParameter` is returned.
    pub fn nvm_get(&mut self, addr: u16) -> RnResult<u8, E> {
        if addr < 0x300 || addr > 0x3ff {
            return Err(Error::BadParameter);
        }

        let mut hex_addr_buf = [0; 4];
        let addr_buf_byte_count = base16::encode_config_slice(
            &addr.to_be_bytes(),
            base16::EncodeLower,
            &mut hex_addr_buf,
        );
        let hex_addr = from_utf8(&hex_addr_buf[..addr_buf_byte_count]).unwrap();

        let response = self.send_raw_command(&["sys get nvm ", utils::ltrim_hex(&hex_addr)])?;
        if response.len() != 2 {
            return Err(Error::ParsingError);
        }
        let mut buf = [0; 1];
        base16::decode_slice(response, &mut buf).map_err(|_| Error::ParsingError)?;
        Ok(buf[0])
    }

    /// Put the system to sleep (with millisecond precision).
    ///
    /// The module can be forced to exit from sleep by sending a break
    /// condition followed by a 0x55 character at the new baud rate.
    ///
    /// **Note:** This command is asynchronous, it will *not* wait for the module
    /// to wake up. You need to call [`wait_for_wakeup()`][wait_for_wakeup] to
    /// wait for the module before sending any other command.
    ///
    /// [wait_for_wakeup]: #method.wait_for_wakeup
    pub fn sleep(&mut self, duration: Duration) -> RnResult<(), E> {
        // Split duration into seconds and milliseconds
        let secs: u64 = duration.as_secs();
        let subsec_millis: u32 = duration.subsec_millis();

        // Millis must be in the range [100, 2^32).
        // Do this the awkward way to avoid using the u128 type that `as_millis` returns.
        let millis: u32 = if secs == 0 && subsec_millis < 100 {
            return Err(Error::BadParameter);
        } else if (secs < 4_294_967) || (secs == 4_294_967 && subsec_millis <= 295) {
            (secs * 1000) as u32 + duration.subsec_millis()
        } else {
            return Err(Error::BadParameter);
        };

        let mut buf = [0u8; 10];
        self.send_raw_command_nowait(&["sys sleep ", millis.numtoa_str(10, &mut buf)])?;
        self.sleep = true;
        Ok(())
    }

    /// After [sleep mode][sleep] has been enabled, wait for wakeup and clear
    /// the `sleep` flag.
    ///
    /// If a sleep is in progress, this will block until the module sends a
    /// line on the serial bus.
    ///
    /// If the `sleep` flag is not set, then the method will return immediately
    /// without a serial read unless the `force` flag is set to `true`. This is
    /// required if you create a new driver instance for a module that is still
    /// in sleep mode.
    ///
    /// **Note:** If the module responds with a response that is not the string
    /// `"ok"`, a [`Error::ParsingError`][parsing-error] will be returned, but
    /// the `sleep` flag will still be cleared (since the module is obviously
    /// not in sleep mode anymore).
    ///
    /// [sleep]: #method.sleep
    /// [parsing-error]: errors/enum.Error.html#variant.ParsingError
    pub fn wait_for_wakeup(&mut self, force: bool) -> RnResult<(), E> {
        // If no sleep is in progress, return immediately
        if !force && !self.sleep {
            return Ok(());
        }

        // Wait for "ok" response.
        // If any response is returned, the `sleep` flag will be cleared.
        match self.read_line()? {
            b"ok" => {
                self.sleep = false;
                Ok(())
            }
            _ => {
                self.sleep = false;
                Err(Error::ParsingError)
            }
        }
    }
}

/// Macro to generate setters and getters for MAC parameters.
macro_rules! hex_setter_getter {
    (
        $field:expr, $bytes:expr, $descr:expr,
        $set_hex:ident, $set_slice:ident
    ) => {
        doc_comment! {
            concat!(
                "Set ",
                $descr,
                ".",
                "\n\nThe parameter must be a ", stringify!($bytes), "-byte hex string, ",
                "otherwise `Error::BadParameter` will be returned.",
            ),
            pub fn $set_hex(&mut self, val: &str) -> RnResult<(), E> {
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
            pub fn $set_slice(&mut self, val: &[u8]) -> RnResult<(), E> {
                if val.len() != $bytes {
                    return Err(Error::BadParameter);
                }
                let mut buf = [0; $bytes * 2];
                base16::encode_config_slice(val, base16::EncodeLower, &mut buf);
                self.$set_hex(from_utf8(&buf)?)
            }
        }
    };
    (
        $field:expr, $bytes:expr, $descr:expr,
        $set_hex:ident, $set_slice:ident,
        $get_hex:ident, $get_slice:ident,
        $(,)?
    ) => {
        hex_setter_getter!($field, $bytes, $descr, $set_hex, $set_slice);

        doc_comment! {
            concat!("Get ", $descr, " as hex str."),
            pub fn $get_hex(&mut self) -> RnResult<&str, E> {
                self.send_raw_command_str(&[concat!("mac get ", $field)])
            }
        }

        doc_comment! {
            concat!("Get ", $descr, " bytes."),
            pub fn $get_slice(&mut self) -> RnResult<[u8; $bytes], E> {
                let hex = self.$get_hex()?;
                let mut buf = [0; $bytes];
                base16::decode_slice(hex, &mut buf).map_err(|_| Error::ParsingError)?;
                Ok(buf)
            }
        }
    };

    // Allow trailing commas
    ($field:expr, $bytes:expr, $descr:expr, $set_hex:ident, $set_slice:ident,) => {
        hex_setter_getter!($field, $bytes, $descr, $set_hex, $set_slice);
    };
    ($field:expr, $bytes:expr, $descr:expr, $set_hex:ident, $set_slice:ident, $get_hex:ident, $get_slice:ident,) => {
        hex_setter_getter!($field, $bytes, $descr, $set_hex, $set_slice, $get_hex, $get_slice);
    };
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
    /// `band`, `deveui`, `appeui`, `appkey`, `nwkskey`, `appskey`, `devaddr`
    /// as well as all channel parameters (e.g. frequeny, duty cycle, data).
    pub fn save_config(&mut self) -> RnResult<(), E> {
        self.send_raw_command_ok(&["mac save"])
    }

    hex_setter_getter!(
        "devaddr",
        4,
        "the unique network device address",
        set_dev_addr_hex,
        set_dev_addr_slice,
        get_dev_addr_hex,
        get_dev_addr_slice,
    );

    hex_setter_getter!(
        "deveui",
        8,
        "the globally unique device identifier",
        set_dev_eui_hex,
        set_dev_eui_slice,
        get_dev_eui_hex,
        get_dev_eui_slice,
    );

    hex_setter_getter!(
        "appeui",
        8,
        "the globally unique application identifier",
        set_app_eui_hex,
        set_app_eui_slice,
        get_app_eui_hex,
        get_app_eui_slice,
    );

    hex_setter_getter!(
        "nwkskey",
        16,
        "the network session key",
        set_network_session_key_hex,
        set_network_session_key_slice,
    );

    hex_setter_getter!(
        "appskey",
        16,
        "the application session key",
        set_app_session_key_hex,
        set_app_session_key_slice,
    );

    hex_setter_getter!(
        "appkey",
        16,
        "the application key",
        set_app_key_hex,
        set_app_key_slice,
    );

    /// Set whether the ADR (adaptive data rate) mechanism is enabled.
    pub fn set_adr(&mut self, enabled: bool) -> RnResult<(), E> {
        let state = if enabled { "on" } else { "off" };
        self.send_raw_command_ok(&["mac set adr ", state])
    }

    /// Return whether the ADR (adaptive data rate) mechanism is enabled.
    pub fn get_adr(&mut self) -> RnResult<bool, E> {
        match self.send_raw_command_str(&["mac get adr"])? {
            "on" => Ok(true),
            "off" => Ok(false),
            _ => Err(Error::ParsingError),
        }
    }

    /// Set the up frame counter.
    pub fn set_upctr(&mut self, upctr: u32) -> RnResult<(), E> {
        let mut buf = [0u8; 10];
        self.send_raw_command_ok(&["mac set upctr ", upctr.numtoa_str(10, &mut buf)])
    }

    /// Get the up frame counter.
    pub fn get_upctr(&mut self) -> RnResult<u32, E> {
        let ctr = self.send_raw_command_str(&["mac get upctr"])?;
        ctr.parse().map_err(|_| Error::ParsingError)
    }

    /// Set the down frame counter.
    pub fn set_dnctr(&mut self, dnctr: u32) -> RnResult<(), E> {
        let mut buf = [0u8; 10];
        self.send_raw_command_ok(&["mac set dnctr ", dnctr.numtoa_str(10, &mut buf)])
    }

    /// Get the down frame counter.
    pub fn get_dnctr(&mut self) -> RnResult<u32, E> {
        let ctr = self.send_raw_command_str(&["mac get dnctr"])?;
        ctr.parse().map_err(|_| Error::ParsingError)
    }

    /// Join the network.
    pub fn join(&mut self, mode: JoinMode) -> Result<(), JoinError<E>> {
        let mode_str = match mode {
            JoinMode::Otaa => "otaa",
            JoinMode::Abp => "abp",
        };

        // First response is whether the join procedure was initialized properly.
        match self.send_raw_command_str(&["mac join ", mode_str])? {
            "ok" => {}
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
    ) -> Result<Option<Downlink>, TxError<E>> {
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
            b"ok" => {}
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
            b"invalid_data_len" => Err(TxError::InvalidDataLenth),
            val if val.starts_with(b"mac_rx ") => {
                let mut parts = from_utf8(val)?.split_ascii_whitespace();

                // Get port
                let _ = parts.next().ok_or(TxError::Other(Error::ParsingError))?;
                let port_str = parts.next().ok_or(TxError::Other(Error::ParsingError))?;
                let port =
                    u8::from_str(&port_str).map_err(|_| TxError::Other(Error::ParsingError))?;
                utils::validate_port(port, TxError::Other(Error::ParsingError))?;

                // Get data
                let hexdata = parts.next().ok_or(TxError::Other(Error::ParsingError))?;
                if hexdata.len() % 2 != 0 {
                    return Err(TxError::Other(Error::ParsingError));
                }

                Ok(Some(Downlink { port, hexdata }))
            }
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
    ) -> Result<Option<Downlink>, TxError<E>> {
        let mut buf = [0; 256];
        let bytes = base16::encode_config_slice(data, base16::EncodeLower, &mut buf);
        self.transmit_hex(mode, port, from_utf8(&buf[0..bytes])?)
    }
}

/// MAC commands for 433 MHz modules.
impl<S, E> Driver<Freq433, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    /// Set the data rate to be used for the following transmissions.
    pub fn set_data_rate(&mut self, data_rate: DataRateEuCn) -> RnResult<(), E> {
        self.send_raw_command_ok(&["mac set dr ", data_rate.into()])
    }

    /// Return the currently configured data rate.
    pub fn get_data_rate(&mut self) -> RnResult<DataRateEuCn, E> {
        let dr = self.send_raw_command_str(&["mac get dr"])?;
        DataRateEuCn::try_from(dr).map_err(|_| Error::ParsingError)
    }
}

/// MAC commands for 868 MHz modules.
impl<S, E> Driver<Freq868, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    /// Set the data rate to be used for the following transmissions.
    pub fn set_data_rate(&mut self, data_rate: DataRateEuCn) -> RnResult<(), E> {
        self.send_raw_command_ok(&["mac set dr ", data_rate.into()])
    }

    /// Return the currently configured data rate.
    pub fn get_data_rate(&mut self) -> RnResult<DataRateEuCn, E> {
        let dr = self.send_raw_command_str(&["mac get dr"])?;
        DataRateEuCn::try_from(dr).map_err(|_| Error::ParsingError)
    }
}

/// MAC commands for 915 MHz modules.
impl<S, E> Driver<Freq915, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    /// Set the data rate to be used for the following transmissions.
    pub fn set_data_rate(&mut self, data_rate: DataRateUs) -> RnResult<(), E> {
        self.send_raw_command_ok(&["mac set dr ", data_rate.into()])
    }

    /// Return the currently configured data rate.
    pub fn get_data_rate(&mut self) -> RnResult<DataRateUs, E> {
        let dr = self.send_raw_command_str(&["mac get dr"])?;
        DataRateUs::try_from(dr).map_err(|_| Error::ParsingError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use embedded_hal_mock::serial::{Mock as SerialMock, Transaction};
    use embedded_hal_mock::MockError;

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
        assert_eq!(
            rn.set_dev_eui_hex("0004a30b001a55e"),
            Err(Error::BadParameter)
        );
        assert_eq!(
            rn.set_dev_eui_hex("0004a30b001a55edx"),
            Err(Error::BadParameter)
        );
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
        assert!(rn
            .set_dev_eui_slice(&[0x00, 0x04, 0xa3, 0x0b, 0x00, 0x1a, 0x55, 0xed])
            .is_ok());
        mock.done();
    }

    fn _get_dev_eui() -> (SerialMock<u8>, Driver<Freq868, SerialMock<u8>>) {
        let expectations = [
            Transaction::write_many(b"mac get deveui\r\n".as_ref()),
            Transaction::read_many(b"0004a30b001a55ed\r\n"),
        ];
        let mock = SerialMock::new(&expectations);
        let rn = rn2483_868(mock.clone());
        (mock, rn)
    }

    #[test]
    fn get_dev_eui_hex() {
        let (mut mock, mut rn) = _get_dev_eui();
        let deveui = rn.get_dev_eui_hex().unwrap();
        assert_eq!(deveui, "0004a30b001a55ed");
        mock.done();
    }

    #[test]
    fn get_dev_eui_slice() {
        let (mut mock, mut rn) = _get_dev_eui();
        let deveui = rn.get_dev_eui_slice().unwrap();
        assert_eq!(deveui, [0x00, 0x04, 0xa3, 0x0b, 0x00, 0x1a, 0x55, 0xed]);
        mock.done();
    }

    mod data_rate {
        use super::*;

        #[test]
        fn set_sf9_eucn() {
            let expectations = [
                Transaction::write_many(b"mac set dr 3\r\n"),
                Transaction::read_many(b"ok\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert!(rn.set_data_rate(DataRateEuCn::Sf9Bw125).is_ok());
            mock.done();
        }

        #[test]
        fn set_sf9_us() {
            let expectations = [
                Transaction::write_many(b"mac set dr 1\r\n"),
                Transaction::read_many(b"ok\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2903_915(mock.clone());
            assert!(rn.set_data_rate(DataRateUs::Sf9Bw125).is_ok());
            mock.done();
        }

        #[test]
        fn set_sf12_eucn() {
            let expectations = [
                Transaction::write_many(b"mac set dr 0\r\n"),
                Transaction::read_many(b"ok\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert!(rn.set_data_rate(DataRateEuCn::Sf12Bw125).is_ok());
            mock.done();
        }

        #[test]
        fn get_sf7_us() {
            let expectations = [
                Transaction::write_many(b"mac get dr\r\n"),
                Transaction::read_many(b"4\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2903_915(mock.clone());
            assert_eq!(rn.get_data_rate().unwrap(), DataRateUs::Sf8Bw500);
            mock.done();
        }
    }

    mod adr {
        use super::*;

        #[test]
        fn get_on() {
            let expectations = [
                Transaction::write_many(b"mac get adr\r\n"),
                Transaction::read_many(b"on\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert!(rn.get_adr().unwrap());
            mock.done();
        }

        #[test]
        fn get_off() {
            let expectations = [
                Transaction::write_many(b"mac get adr\r\n"),
                Transaction::read_many(b"off\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert!(!rn.get_adr().unwrap());
            mock.done();
        }

        #[test]
        fn get_invalid() {
            let expectations = [
                Transaction::write_many(b"mac get adr\r\n"),
                Transaction::read_many(b"of\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.get_adr().unwrap_err(), Error::ParsingError);
            mock.done();
        }

        #[test]
        fn set() {
            let expectations = [
                Transaction::write_many(b"mac set adr on\r\n"),
                Transaction::read_many(b"ok\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert!(rn.set_adr(true).is_ok());
            mock.done();
        }
    }

    mod sleep {
        use super::*;

        #[test]
        fn sleep_min_max_duration() {
            // Min duration: 100ms
            let expectations = [Transaction::write_many(b"sys sleep 100\r\n")];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert!(rn.sleep(Duration::from_millis(100)).is_ok());
            mock.done();

            // Max duration: (2**32)-1 ms
            let expectations = [Transaction::write_many(b"sys sleep 4294967295\r\n")];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert!(rn.sleep(Duration::from_millis((1 << 32) - 1)).is_ok());
            mock.done();
        }

        #[test]
        fn sleep_invalid_durations() {
            let expectations = [];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());

            assert_eq!(rn.sleep(Duration::from_millis(0)), Err(Error::BadParameter));
            assert_eq!(
                rn.sleep(Duration::from_millis(99)),
                Err(Error::BadParameter)
            );
            assert_eq!(
                rn.sleep(Duration::from_millis(1 << 32)),
                Err(Error::BadParameter)
            );

            mock.done();
        }

        /// While the sleep mode flag is set, don't issue any serial writes.
        #[test]
        fn sleep_mode_no_write() {
            let expectations = [Transaction::write_many(b"sys sleep 1000\r\n")];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());

            // Put device into sleep mode
            rn.sleep(Duration::from_secs(1)).unwrap();

            // A write call should now fail without causing a write transaction
            assert_eq!(rn.write_all(b"123"), Err(Error::SleepMode));
            assert_eq!(rn.write_crlf(), Err(Error::SleepMode));
            mock.done();
        }

        /// Waiting for wakeup will return immediately (without a read) if no
        /// sleep is in progress.
        #[test]
        fn wait_for_wakeup_immediate() {
            // Waiting for wakeup should not cause a read transaction...
            let expectations = [];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.wait_for_wakeup(false), Ok(()));
            assert_eq!(rn.wait_for_wakeup(false), Ok(()));
            assert_eq!(rn.wait_for_wakeup(false), Ok(()));
            mock.done();

            // ...unless the `force` flag is set.
            let expectations = [Transaction::read_many(b"ok\r\n")];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            assert_eq!(rn.wait_for_wakeup(true), Ok(()));
            mock.done();
        }

        /// Waiting for wakeup will handle non-"ok" responses as errors.
        #[test]
        fn wait_for_wakeup_errors() {
            let expectations = [Transaction::read_many(b"ohno\r\n")];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            rn.sleep = true;

            // Parsing the response will return an error
            assert_eq!(rn.wait_for_wakeup(false), Err(Error::ParsingError));

            // But the sleep flag will still be cleared
            assert!(!rn.sleep);

            mock.done();
        }
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
            assert_eq!(
                rn.transmit_hex(ConfirmationMode::Unconfirmed, 42, "23ff"),
                Ok(None)
            );
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
            assert_eq!(
                rn.transmit_hex(ConfirmationMode::Confirmed, 42, "23ff"),
                Ok(None)
            );
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

    mod ensure_known_state {
        use super::*;

        use std::io::ErrorKind;

        #[test]
        fn already_clean() {
            let expectations = [
                // Our initial buffer is empty
                Transaction::read_error(nb::Error::WouldBlock),
                // Expect the 'z' write
                Transaction::write_many(b"z\r\n"),
                // Read returns invalid_param
                Transaction::read_many(b"invalid_param\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            rn.ensure_known_state().unwrap();
            mock.done();
        }

        #[test]
        fn non_empty_buffer() {
            let expectations = [
                // Our initial buffer still contains some bytes
                Transaction::read_many(b"sys "),
                Transaction::read_many(b"reset"),
                Transaction::read_error(nb::Error::WouldBlock),
                // Expect the 'z' write
                Transaction::write_many(b"z\r\n"),
                // Read returns invalid_param
                Transaction::read_many(b"invalid_param\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            rn.ensure_known_state().unwrap();
            mock.done();
        }

        #[test]
        fn retry() {
            let expectations = [
                // Initial buffer empty
                Transaction::read_error(nb::Error::WouldBlock),
                // Expect the 'z' write
                Transaction::write_many(b"z\r\n"),
                // Read returns unexpected data
                Transaction::read_many(b"ok\r\n"),
                // Expect the 'z' write again (attempt 2)
                Transaction::write_many(b"z\r\n"),
                // Still unexpected data
                Transaction::read_many(b"wtf\r\n"),
                // Expect the 'z' write again (attempt 3)
                Transaction::write_many(b"z\r\n"),
                // Finally!
                Transaction::read_many(b"invalid_param\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());
            rn.ensure_known_state().unwrap();
            mock.done();
        }

        #[test]
        fn retry_failed() {
            let expectations = [
                // Initial buffer empty
                Transaction::read_error(nb::Error::WouldBlock),
                // Unexpected response for 3 consecutive attempts
                Transaction::write_many(b"z\r\n"),
                Transaction::read_many(b"uhm\r\n"),
                Transaction::write_many(b"z\r\n"),
                Transaction::read_many(b"lol\r\n"),
                Transaction::write_many(b"z\r\n"),
                Transaction::read_many(b"wat\r\n"),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());

            // Fail after 3 attempts
            assert_eq!(rn.ensure_known_state().unwrap_err(), Error::InvalidState);

            mock.done();
        }

        #[test]
        fn read_error() {
            let expectations = [
                // Read fails with an error
                Transaction::read_error(nb::Error::Other(MockError::Io(ErrorKind::BrokenPipe))),
            ];
            let mut mock = SerialMock::new(&expectations);
            let mut rn = rn2483_868(mock.clone());

            // Errors while reading are propagated
            assert_eq!(
                rn.ensure_known_state().unwrap_err(),
                Error::SerialRead(MockError::Io(ErrorKind::BrokenPipe))
            );

            mock.done();
        }
    }
}
