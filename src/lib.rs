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
}

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
    fn write_byte(&mut self, byte: u8) -> Result<(), Error> {
        block!(self.serial.write(byte)).map_err(|_| Error::SerialWrite)
    }

    /// Write all bytes from the buffer to the serial port.
    fn write_all(&mut self, buffer: &[u8], crlf: bool) -> Result<(), Error> {
        for byte in buffer {
            self.write_byte(*byte)?;
        }
        if crlf {
            self.write_byte(CR)?;
            self.write_byte(LF)?;
        }
        Ok(())
    }

    /// Read a single byte from the serial port.
    fn read_byte(&mut self) -> Result<u8, Error> {
        block!(self.serial.read()).map_err(|_| Error::SerialRead)
    }

    /// Read a CR/LF terminated line from the serial port.
    ///
    /// The string is returned without the line termination.
    fn read_line(&mut self) -> Result<String, Error> {
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
    pub fn send_raw_command(&mut self, command: &str) -> Result<String, Error> {
        self.write_all(command.as_bytes(), true)?;
        self.read_line()
    }
}

/// Query system information.
impl<S, E> Rn2xx3<S>
where
    S: serial::Read<u8, Error = E> + serial::Write<u8, Error = E>,
{
    /// Return the preprogrammed EUI node address as uppercase hex string.
    pub fn hweui(&mut self) -> Result<String, Error> {
        self.send_raw_command("sys get hweui")
    }

    /// Return the version string.
    pub fn version(&mut self) -> Result<String, Error> {
        self.send_raw_command("sys get ver")
    }

    /// Return the model of the module.
    pub fn model(&mut self) -> Result<Model, Error> {
        let version = self.version()?;
        match &version[0..6] {
            "RN2483" => Ok(Model::RN2483),
            "RN2903" => Ok(Model::RN2903),
            _ => Err(Error::ParsingError),
        }
    }

    /// Measure and return the Vdd voltage in millivolts.
    pub fn vdd(&mut self) -> Result<u16, Error> {
        let vdd = self.send_raw_command("sys get vdd")?;
        vdd.parse().map_err(|_| Error::ParsingError)
    }
}
