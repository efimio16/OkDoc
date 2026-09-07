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
        const KW_TRUE: &str = "true";
        const KW_FALSE: &str = "false";
        
        let mut value = None;
        
        let chunk = input.fill_buf()?;

        if value.is_none() {
            let first_char = chunk[0];
            match first_char {
                b't' => value = Some(true),
                b'f' => value = Some(false),
                _ => return Err(PdfError::Parse(format!("invalid first character for boolean: {}", first_char as char)))
            }
        }

        match value {
            Some(true) => {
                input.read_keyword(KW_TRUE)?;
                Ok(Self(true))
            }
            Some(false) => {
                input.read_keyword(KW_FALSE)?;
                Ok(Self(false))
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use super::*;

    fn assert_slice_and_value(slice: &[u8], value: PdfBoolean) {
        assert_eq!(PdfBoolean::from_bytes(slice).unwrap(), value);
    }

    fn assert_err(slice: &[u8]) {
        assert_matches!(PdfBoolean::from_bytes(slice), Err(_));
    }

    #[test]
    fn test_boolean() {
        assert_slice_and_value(b"true", PdfBoolean(true));
        assert_slice_and_value(b"false", PdfBoolean(false));

        assert_slice_and_value(b"true lorem ipsum", PdfBoolean(true));
        assert_slice_and_value(b"false lorem ipsum", PdfBoolean(false));

        assert_err(b"trua");
        assert_err(b" false");
        assert_err(b"fal se");
    }
}