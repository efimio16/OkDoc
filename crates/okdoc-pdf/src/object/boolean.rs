use std::io::BufRead;

use crate::{error::PdfError, parser::{Parseable, PdfInput}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PdfBoolean(pub bool);

impl Parseable for PdfBoolean {
    fn matches_from_start(bytes: &[u8]) -> usize {
        // const TRUE: &[u8] = b"true";
        // const FALSE: &[u8] = b"false";

        // if bytes[0] == t
        // for (i, b) in bytes.iter().enumerate() {
        //     if TRUE.get(i) != Some(b) && FALSE.get(i) != Some(b) {
        //         return i;
        //     }
        // }

        bytes.len()
    }

    fn from_bytes<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        const TRUE: &str = "true";
        const FALSE: &str = "false";
        
        let mut value = None;
        
        let chunk = input.fill_buf()?;

        if let None = value {
            let first_char = chunk[0];
            match first_char {
                b't' => value = Some(true),
                b'f' => value = Some(false),
                _ => return Err(PdfError::Parse(format!("invalid first character for boolean: {}", first_char as char)))
            }
        }

        match value {
            Some(true) => {
                input.read_keyword(TRUE)?;
                Ok(Self(true))
            }
            Some(false) => {
                input.read_keyword(FALSE)?;
                Ok(Self(false))
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean() {
        assert_eq!(PdfBoolean::from_bytes(b"true" as &[u8]).unwrap().0, true);
        assert_eq!(PdfBoolean::from_bytes(b"false" as &[u8]).unwrap().0, false);
    }
}