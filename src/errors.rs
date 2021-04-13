//! Error types used in this driver.

use core::str::Utf8Error;

/// A collection of errors that can occur.
#[derive(Debug, PartialEq, Eq)]
pub enum Error<S> {
    /// Could not read from serial port.
    SerialRead(S),
    /// Could not write to serial port.
    SerialWrite(S),
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
    /// Device is in sleep mode.
    SleepMode,
    /// The module is in an invalid state.
    InvalidState,
}

impl<S> From<Utf8Error> for Error<S> {
    fn from(_: Utf8Error) -> Self {
        Error::EncodingError
    }
}

/// Errors that can occur during the join procedure.
#[derive(Debug, PartialEq, Eq)]
pub enum JoinError<S> {
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
    Other(Error<S>),
}

impl<S> From<Error<S>> for JoinError<S> {
    fn from(other: Error<S>) -> Self {
        JoinError::Other(other)
    }
}

impl<S> From<Utf8Error> for JoinError<S> {
    fn from(_: Utf8Error) -> Self {
        JoinError::Other(Error::EncodingError)
    }
}

/// Errors that can occur during the transmit procedure.
#[derive(Debug, PartialEq, Eq)]
pub enum TxError<S> {
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
    Other(Error<S>),
}

impl<S> From<Error<S>> for TxError<S> {
    fn from(other: Error<S>) -> Self {
        TxError::Other(other)
    }
}

impl<S> From<Utf8Error> for TxError<S> {
    fn from(_: Utf8Error) -> Self {
        TxError::Other(Error::EncodingError)
    }
}

/// A `Result<T, Error>`.
pub type RnResult<T, S> = Result<T, Error<S>>;
