use core::str::from_utf8;

use base16;
use embedded_hal::serial;
use nb::block;

const CR: u8 = 0x0d;
const LF: u8 = 0x0a;

/// The driver instance for both RN2483 and RN2903.
pub struct Rn2xx3<S> {
    serial: S,
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
}

/// A `Result<T, Error>`.
pub type RnResult<T> = Result<T, Error>;

/// List of all supported RN module models.
#[derive(Debug)]
pub enum Model {
    RN2483,
    RN2903,
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(_: std::string::FromUtf8Error) -> Self {
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
        Self { serial }
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
    fn read_line(&mut self) -> RnResult<String> {
        let mut buf: Vec<u8> = vec![];
        let mut cr_read = false;
        loop {
            match self.read_byte()? {
                CR => {
                    cr_read = true;
                    buf.push(CR);
                }
                LF if cr_read => {
                    // Remove CR
                    buf.remove(buf.len() - 1);
                    return Ok(String::from_utf8(buf)?);
                }
                other => {
                    cr_read = false;
                    buf.push(other);
                }
            }
        }
    }

    /// Send a raw command to the module and return the response.
    pub fn send_raw_command(&mut self, command: &[&str]) -> RnResult<String> {
        for part in command {
            self.write_all(part.as_bytes())?;
        }
        self.write_crlf()?;
        self.read_line()
    }

    /// Send a raw command that should be confirmed with 'OK'. If the response
    /// is not 'OK', return `Error::CommandFailed`.
    fn send_raw_ok_command(&mut self, command: &[&str]) -> RnResult<()> {
        match &*self.send_raw_command(command)? {
            "ok" => Ok(()),
            _ => Err(Error::CommandFailed),
        }
    }
}

/// System commands.
impl<S, E> Rn2xx3<S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    /// Reset and restart the RN module. Return the version string.
    pub fn reset(&mut self) -> RnResult<String> {
        self.send_raw_command(&["sys reset"])
    }

    /// Reset the module's configuration data and user EEPROM to factory
    /// default values and restart the module.
    ///
    /// All configuration parameters will be restored to factory default
    /// values. Return the version string.
    pub fn factory_reset(&mut self) -> RnResult<String> {
        self.send_raw_command(&["sys factoryRESET"])
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
    pub fn hweui(&mut self) -> RnResult<String> {
        self.send_raw_command(&["sys get hweui"])
    }

    /// Return the version string.
    pub fn version(&mut self) -> RnResult<String> {
        self.send_raw_command(&["sys get ver"])
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
        let vdd = self.send_raw_command(&["sys get vdd"])?;
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
        self.send_raw_ok_command(&args)
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
        let hex_byte = self.send_raw_command(&["sys get nvm ", &hex_addr])?;
        if hex_byte.len() != 2 {
            return Err(Error::ParsingError);
        }
        let mut buf = [0; 1];
        base16::decode_slice(hex_byte.as_bytes(), &mut buf)
            .map_err(|_| Error::ParsingError)?;
        Ok(buf[0])
    }
}
