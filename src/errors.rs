use core::str::Utf8Error;

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
    /// Command or response contained invalid UTF-8.
    EncodingError,
    /// A response could not be parsed.
    ParsingError,
    /// A command failed.
    CommandFailed,
    /// A bad parameter was supplied.
    BadParameter,
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error::EncodingError
    }
}

/// Errors that can occur during the join procedure.
#[derive(Debug, PartialEq, Eq)]
pub enum JoinError {
    /// Invalid join mode. This indicates a bug in the driver and should be
    /// reported on GitHub.
    BadParameter,
    /// The keys corresponding to the join mode (OTAA or ABP) were not
    /// configured.
    KeysNotInit,
    /// All channels are busy.
    NoFreeChannel,
    /// Device is in a Silent Immediately state.
    Silent,
    /// MAC state is not idle.
    Busy,
    /// MAC was paused and not resumed.
    MacPaused,
    /// Join procedure was unsuccessful: Device tried to join but was rejected
    /// or did not receive a response.
    JoinUnsuccessful,
    /// Unknown response.
    UnknownResponse,
    /// Another error occurred.
    Other(Error),
}

impl From<Error> for JoinError {
    fn from(other: Error) -> Self {
        JoinError::Other(other)
    }
}

impl From<Utf8Error> for JoinError {
    fn from(_: Utf8Error) -> Self {
        JoinError::Other(Error::EncodingError)
    }
}

/// Errors that can occur during the transmit procedure.
#[derive(Debug, PartialEq, Eq)]
pub enum TxError {
    /// Invalid type, port or data.
    BadParameter,
    /// Network not joined.
    NotJoined,
    /// All channels are busy.
    NoFreeChannel,
    /// Device is in a Silent Immediately state.
    Silent,
    /// Frame counter rollover. Re-join needed.
    FrameCounterRollover,
    /// MAC state is not idle.
    Busy,
    /// MAC was paused and not resumed.
    MacPaused,
    /// Application payload length is greater than the maximum application
    /// payload length corresponding to the current data rate.
    InvalidDataLenth,
    /// Transmission was not successful.
    TxUnsuccessful,
    /// Unknown response.
    UnknownResponse,
    /// Another error occurred.
    Other(Error),
}

impl From<Error> for TxError {
    fn from(other: Error) -> Self {
        TxError::Other(other)
    }
}

impl From<Utf8Error> for TxError {
    fn from(_: Utf8Error) -> Self {
        TxError::Other(Error::EncodingError)
    }
}

/// A `Result<T, Error>`.
pub type RnResult<T> = Result<T, Error>;
