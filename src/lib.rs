use core::str::{from_utf8, Utf8Error};

use base16;
use embedded_hal::serial;
use nb::block;

const CR: u8 = 0x0d;
const LF: u8 = 0x0a;
const OK: [u8; 2] = [b'o', b'k'];

/// The driver instance for both RN2483 and RN2903.
pub struct Rn2xx3<S> {
    serial: S,
    read_buf: [u8; 64],
}

/// A collection of all errors that can occur.
#[derive(Debug)]
pub enum Error {
    /// Could not read from serial port.
    SerialRead,
    /// Could not write to serial port.
    SerialWrite,
    /// Command contained invalid UTF-8.
    EncodingError,
    /// A response could not be parsed.
    ParsingError,
    /// A command failed.
    CommandFailed,
    /// Bad address.
    BadAddress,
    /// Read buffer is too small.
    /// This is a bug, please report it on GitHub!
    ReadBufferTooSmall,
}

/// A `Result<T, Error>`.
pub type RnResult<T> = Result<T, Error>;

/// List of all supported RN module models.
#[derive(Debug)]
pub enum Model {
    RN2483,
    RN2903,
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::EncodingError
    }
}

/// Basic commands.
impl<S, E> Rn2xx3<S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    /// Create a new driver, wrapping the specified serial port.
    pub fn new(serial: S) -> Self {
        Self { serial, read_buf: [0; 64] }
    }

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
                    return Ok(&self.read_buf[0..i-1]);
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
impl<S, E> Rn2xx3<S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
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
    /// `Error::BadAddress` is returned.
    pub fn nvm_set(&mut self, addr: u16, byte: u8) -> RnResult<()> {
        if addr < 0x300 || addr > 0x3ff {
            return Err(Error::BadAddress);
        }
        let hex_addr = format!("{:x}", addr);
        let (h, l) = base16::encode_byte_l(byte);
        let hex_byte_bytes = [h, l];
        let hex_byte = from_utf8(&hex_byte_bytes).unwrap();
        let args = ["sys set nvm ", &hex_addr, " ", &hex_byte];
        self.send_raw_command_ok(&args)
    }

    /// Get the NVM byte at `addr`.
    ///
    /// The address must be between 0x300 and 0x3ff, otherwise
    /// `Error::BadAddress` is returned.
    pub fn nvm_get(&mut self, addr: u16) -> RnResult<u8> {
        if addr < 0x300 || addr > 0x3ff {
            return Err(Error::BadAddress);
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
