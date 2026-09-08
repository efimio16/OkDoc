use std::io::{self, BufRead};

use crate::{error::PdfError, parseable::{Parseable, PdfInput}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PdfBoolean(pub bool);

impl Parseable for PdfBoolean {
    fn matches(_: &[u8]) -> usize {
        // const TRUE: &[u8] = b"true";
        // const FALSE: &[u8] = b"false";

        // if bytes[0] == t
        // for (i, b) in bytes.iter().enumerate() {
        //     if TRUE.get(i) != Some(b) && FALSE.get(i) != Some(b) {
        //         return i;
        //     }
        // }

        todo!("decide if this function is still needed")
    }

    fn parse_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        const KW_TRUE: &str = "true";
        const KW_FALSE: &str = "false";
        
        let mut value = None;
        
        let chunk = input.fill_buf()?;
        if chunk.len() == 0 {
            return Err(PdfError::Io(io::ErrorKind::UnexpectedEof.into()));
        }

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
    use crate::parseable::test_utils::{assert_err, assert_slice_and_value};
    use super::*;

    #[test]
    fn test_boolean() {
        assert_slice_and_value(b"true", PdfBoolean(true));
        assert_slice_and_value(b"false", PdfBoolean(false));

        assert_slice_and_value(b"true lorem ipsum", PdfBoolean(true));
        assert_slice_and_value(b"false lorem ipsum", PdfBoolean(false));

        assert_err::<PdfBoolean>(b"trua");
        assert_err::<PdfBoolean>(b" false");
        assert_err::<PdfBoolean>(b"fals");
        assert_err::<PdfBoolean>(b"tru");
    }
}