use core::marker::PhantomData;
use core::str::{from_utf8, Utf8Error};

use base16;
use doc_comment::doc_comment;
use embedded_hal::serial;
use nb::block;

const CR: u8 = 0x0d;
const LF: u8 = 0x0a;
const OK: [u8; 2] = [b'o', b'k'];

/// Marker trait implemented for all model type parameters.
pub trait ModelParam {}
/// Model type parameter for the RN2483 (433 MHz).
pub struct Freq433;
/// Model type parameter for the RN2483 (868 MHz).
pub struct Freq868;
/// Model type parameter for the RN2903 (915 MHz).
pub struct Freq915;
impl ModelParam for Freq433 {}
impl ModelParam for Freq868 {}
impl ModelParam for Freq915 {}

/// The main driver instance.
pub struct Driver<M: ModelParam, S> {
    model: PhantomData<M>,
    serial: S,
    read_buf: [u8; 64],
}

/// A collection of all errors that can occur.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// Could not read from serial port.
    SerialRead,
    /// Could not write to serial port.
    SerialWrite,
    /// Read buffer is too small.
    /// This is a bug, please report it on GitHub!
    ReadBufferTooSmall,
    /// Command contained invalid UTF-8.
    EncodingError,
    /// A response could not be parsed.
    ParsingError,
    /// A command failed.
    CommandFailed,
    /// A bad parameter was supplied.
    BadParameter,
}

/// A `Result<T, Error>`.
pub type RnResult<T> = Result<T, Error>;

/// List of all supported RN module models.
#[derive(Debug, PartialEq, Eq)]
pub enum Model {
    RN2483,
    RN2903,
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::EncodingError
    }
}

/// Create a new driver instance for the RN2483 (433 MHz), wrapping the
/// specified serial port.
pub fn rn2483_433<S, E>(serial: S) -> Driver<Freq433, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    Driver {
        model: PhantomData,
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
        model: PhantomData,
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
        model: PhantomData,
        serial,
        read_buf: [0; 64],
    }
}

/// Basic commands.
impl<M, S, E> Driver<M, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    M: ModelParam,
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
impl<M, S, E> Driver<M, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    M: ModelParam,
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
        let (h, l) = base16::encode_byte_l(byte);
        let hex_byte_bytes = [h, l];
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

macro_rules! hex_setter {
    ($field:expr, $bytes:expr, $descr:expr, $set_hex:ident, $set_slice:ident, $(,)?) => {
        doc_comment! {
            concat!(
                $descr,
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
                $descr,
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
    }
}

/// MAC Set Commands.
impl<M, S, E> Driver<M, S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
    M: ModelParam,
{
    hex_setter!(
        "devaddr", 4,
        "Set the unique network device address.",
        set_dev_addr_hex,
        set_dev_addr_slice,
    );

    hex_setter!(
        "deveui", 8,
        "Set the globally unique device identifier.",
        set_dev_eui_hex,
        set_dev_eui_slice,
    );

    hex_setter!(
        "appeui", 8,
        "Set the globally unique application identifier.",
        set_app_eui_hex,
        set_app_eui_slice,
    );

    hex_setter!(
        "nwkskey", 16,
        "Set the network session key.",
        set_network_session_key_hex,
        set_network_session_key_slice,
    );

    hex_setter!(
        "appskey", 16,
        "Set the application session key.",
        set_app_session_key_hex,
        set_app_session_key_slice,
    );

    hex_setter!(
        "appkey", 16,
        "Set the application key.",
        set_app_key_hex,
        set_app_key_slice,
    );
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

}
