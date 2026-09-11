use std::io::{BufRead, ErrorKind::UnexpectedEof};

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

        todo!()
    }

    fn parse_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        const KW_TRUE: &str = "true";
        const KW_FALSE: &str = "false";
        
        let chunk = input.fill_buf()?;
        if chunk.len() == 0 {
            return Err(PdfError::Io(UnexpectedEof.into()));
        }

        let value;
        let first_char = chunk[0];
        match first_char {
            b't' => value = true,
            b'f' => value = false,
            _ => return Err(PdfError::Parse(format!("invalid first character for boolean: {}", first_char as char)))
        }

        input.read_keyword(if value { KW_TRUE } else { KW_FALSE })?;

        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use crate::parseable::test_utils::{assert_err, assert_parsing};
    use super::*;

    #[test]
    fn test_boolean() {
        assert_parsing(b"true", PdfBoolean(true), b"");
        assert_parsing(b"false", PdfBoolean(false), b"");

        assert_parsing(b"true lorem ipsum", PdfBoolean(true), b" lorem ipsum");
        assert_parsing(b"false% A comment", PdfBoolean(false), b"% A comment");

        assert_err::<PdfBoolean>(b"trua");
        assert_err::<PdfBoolean>(b" false");
        assert_err::<PdfBoolean>(b"fals");
        assert_err::<PdfBoolean>(b"trueaeaeuau");
    }
}