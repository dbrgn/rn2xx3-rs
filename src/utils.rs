use core::str::from_utf8;

use crate::errors::{Error, RnResult};

/// Convert a byte to a decimal string.
///
/// If the buffer is too small, `Error::BadParameter` is returned.
pub(crate) fn u8_to_str(val: u8, buf: &mut [u8]) -> RnResult<&str> {
    let chars = match val {
        0..=9 => 1,
        10..=99 => 2,
        100..=255 => 3,
    };
    if buf.len() < chars {
        return Err(Error::BadParameter);
    }
    for i in 0..chars {
        let pos = chars - i - 1;
        let modulo = val as usize % (10usize.pow(i as u32 + 1));
        let digit = modulo / 10usize.pow(i as u32);
        let ascii = digit + 48;
        buf[pos] = ascii as u8;
    }
    Ok(from_utf8(&buf[0..chars])?)
}

/// Trim leading zeroes from a hex string.
///
/// Examples:
///
/// - "1234" -> "1234"
/// - "0123" -> "123"
///
/// All-zero values will be returned as a single zero:
///
/// - "000" -> "0"
///
/// Empty values will remain empty:
///
/// - "" -> ""
///
/// Non-hex characters will be ignored:
///
/// - "zzz" -> "zzz"
pub(crate) fn ltrim_hex(val: &str) -> &str {
    if val.is_empty() {
        return val;
    }
    let trimmed = val.trim_start_matches('0');
    if trimmed.is_empty() {
        "0"
    } else {
        trimmed
    }
}

pub(crate) fn validate_port<T>(port: u8, err: T) -> Result<(), T> {
    if port >= 1 && port <= 223 {
        Ok(())
    } else {
        Err(err)
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    mod u8_to_str {
        use super::*;

        #[test]
        fn all_digits() {
            let mut buf = [0; 3];
            for i in 0..=255 {
                let string = u8_to_str(i, &mut buf).unwrap();
                assert_eq!(string, &std::format!("{}", i));
            }
        }

        #[test]
        fn buf_too_small() {
            let mut buf1 = [0; 1];
            let mut buf2 = [0; 2];
            let mut buf3 = [0; 3];

            assert!(u8_to_str(9, &mut buf1).is_ok());
            assert_eq!(u8_to_str(10, &mut buf1), Err(Error::BadParameter));

            assert!(u8_to_str(99, &mut buf2).is_ok());
            assert_eq!(u8_to_str(100, &mut buf2), Err(Error::BadParameter));

            assert!(u8_to_str(255, &mut buf3).is_ok());
            assert_eq!(u8_to_str(255, &mut buf2), Err(Error::BadParameter));
        }
    }

    mod ltrim_hex {
        use super::*;

        #[test]
        fn no_prefix() {
            assert_eq!(ltrim_hex("1234"), "1234");
        }

        #[test]
        fn single_prefix() {
            assert_eq!(ltrim_hex("0123"), "123");
        }

        #[test]
        fn multi_prefix() {
            assert_eq!(ltrim_hex("0001"), "1");
        }

        #[test]
        fn zero() {
            assert_eq!(ltrim_hex("0000"), "0");
        }

        #[test]
        fn empty() {
            assert_eq!(ltrim_hex(""), "");
        }

        #[test]
        fn nonhex() {
            assert_eq!(ltrim_hex("zzz"), "zzz");
        }
    }
}
