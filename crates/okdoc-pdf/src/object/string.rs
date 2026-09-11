use std::io::{BufRead, ErrorKind::UnexpectedEof};

use bytes::{BufMut, Bytes, BytesMut};
use memchr::{memchr, memchr3_iter};

use crate::{error::PdfError, parseable::Parseable};

#[inline(always)]
fn from_hex(v: u8) -> u8 {
    match v {
        b'0'..=b'9' => v - b'0',
        b'A'..=b'F' => v - b'A' + 10,
        b'a'..=b'f' => v - b'a' + 10,
        _ => 0,
    }
}

/// A string object in PDF.
/// 
/// It's not using [`String`], because Rust strings depend on UTF-8, while PDF
/// can have different character encodings.
/// 
/// ## Syntax
/// Literals: `(Strings may contain balanced parentheses ( ) and special characters (*!&}^% and so on).)`
/// 
/// Hexadecimals: `<4E6F762073686D6F7A206B6120706F702E>`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PdfString(pub Bytes);

impl PdfString {
    fn parse_literal_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        #[derive(Default)]
        enum State {
            #[default]
            CopyBytes,
            EscapeChar,
            EscapeSequence { digits_count: u8, sequence: u16 },
            EscapeMaybeEndOfLine,
        }
        
        let mut state = State::default();
        let mut value = BytesMut::new();
        
        // Allows balanced pairs of parentheses (e.g. `(Strings may contain balanced parentheses (like these).)` )
        let mut parentheses_depth = 0;

        loop {
            let chunk = input.fill_buf()?;
            if chunk.len() == 0 {
                return Err(PdfError::Io(UnexpectedEof.into()));
            }
            
            let mut i = 0;
            while i < chunk.len() {
                match &mut state {
                    State::CopyBytes => {
                        if let Some(j) = memchr3_iter(b'(', b')', b'\\', &chunk[i..]).next() {
                            value.put(&chunk[i..i+j]);
                            match chunk[i+j] {
                                b'(' => {
                                    value.put_u8(b'(');
                                    parentheses_depth += 1;
                                }
                                b')' => {
                                    if parentheses_depth == 0 {
                                        input.consume(i+j+1);
                                        return Ok(Self(value.freeze()))
                                    }
                                    value.put_u8(b')');
                                    parentheses_depth -= 1;
                                }
                                b'\\' => {
                                    state = State::EscapeChar;
                                }
                                _ => unreachable!(),
                            }
                            i += j + 1;
                        } else {
                            value.put(&chunk[i..]);
                            i = chunk.len();
                        }
                    }
                    State::EscapeChar => {
                        let b = chunk[i];
                        match b {
                            b'n' => value.put_u8(b'\n'),
                            b'r' => value.put_u8(b'\r'),
                            b't' => value.put_u8(b'\t'),
                            b'b' => value.put_u8(0x8),
                            b'f' => value.put_u8(0xc),
                            b'(' | b')' | b'\\' => value.put_u8(b),
                            b'\n' | b'\r' => {
                                i += 1;
                                state = State::EscapeMaybeEndOfLine;
                                continue;
                            }
                            b'0'..=b'7' => {
                                state = State::EscapeSequence { digits_count: 0, sequence: 0 };
                                continue;
                            }
                            _ => return Err(PdfError::Parse(format!("invalid character to escape: {}", b))),
                        }
                        i += 1;
                        state = State::CopyBytes;
                    }
                    State::EscapeSequence { digits_count, sequence } => {
                        let b = chunk[i];
                        match b {
                            b'0'..=b'7' => {
                                *sequence = *sequence * 8 + (b - b'0') as u16;
                                *digits_count += 1;
                                if *digits_count == 3 {
                                    if *sequence >> 8 > 0 {
                                        value.put_u8((*sequence >> 8) as u8);
                                    }
                                    value.put_u8((*sequence & 0xff) as u8);
                                    state = State::CopyBytes;
                                }
                            }
                            _ => {
                                if *sequence >> 8 > 0 {
                                    value.put_u8((*sequence >> 8) as u8);
                                }
                                value.put_u8((*sequence & 0xff) as u8);
                                state = State::CopyBytes;
                                continue;
                            }
                        }
                        i += 1;
                    }
                    State::EscapeMaybeEndOfLine => {
                        match chunk[i] {
                            b'\n' | b'\r' => i += 1,
                            _ => {}
                        }
                        state = State::CopyBytes;
                    }
                }
            }

            input.consume(i);
        }
    }
    fn parse_hex_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        let mut value = BytesMut::new();
        let mut remaining_byte = None;
        
        loop {
            let chunk = input.fill_buf()?;
            let chunk_len = chunk.len();
            if chunk_len == 0 {
                return Err(PdfError::Io(UnexpectedEof.into()));
            }
            
            let input_end = memchr(b'>', chunk).unwrap_or(chunk_len);
            
            let mut i = 0;

            if let Some(remaining_byte) = remaining_byte {
                value.put_u8(from_hex(remaining_byte) << 4 | from_hex(chunk[0]));
                i += 1;
            }

            let pairs_end = if (input_end - i) % 2 == 1 {
                remaining_byte = Some(chunk[input_end - 1]);
                input_end - 1
            } else {
                input_end
            };
            
            while i < pairs_end {
                value.put_u8(from_hex(chunk[i]) << 4 | from_hex(chunk[i + 1]));
                i += 2;
            }
            
            input.consume(i);

            if input_end < chunk_len {
                input.consume(1);
                return Ok(Self(value.freeze()))
            }
        }
    }
}

impl Parseable for PdfString {
    fn matches(_: &[u8]) -> usize {
        todo!()
    }

    fn parse_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        let chunk = input.fill_buf()?;
        if chunk.len() == 0 {
            return Err(PdfError::Io(UnexpectedEof.into()));
        }

        let first_byte = chunk[0];
        input.consume(1);

        match first_byte {
            // Literal string
            b'(' => Self::parse_literal_from(input),
            // Hex string
            b'<' => Self::parse_hex_from(input),
            _ => return Err(PdfError::Parse(format!("unexpected first character: {}", first_byte as char))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parseable::test_utils::{assert_err, assert_parsing};
    use super::*;

    fn test_parsing(input: &[u8], string: &'static [u8]) {
        assert_parsing(input, PdfString(Bytes::from_static(string)), b"");
    }
    
    #[test]
    fn test_literal_strings() {
        test_parsing(b"(This is a string)", b"This is a string");
        test_parsing(
            b"(Strings may contain newlines\nand such.)",
            b"Strings may contain newlines\nand such."
        );
        test_parsing(
            b"(Strings may contain balanced parentheses ( ) and special characters (*!&}^% and so on).)",
            b"Strings may contain balanced parentheses ( ) and special characters (*!&}^% and so on)."
        );
        test_parsing(
            b"(These \\\ntwo strings \\\nare the same.)",
            b"These two strings are the same."
        );
        test_parsing(
            b"(This string contains \\245two octal characters\\307.)",
            b"This string contains \xa5two octal characters\xc7."
        );
        test_parsing(b"(\\0053)", b"\x053");
        test_parsing(b"(\\053)", b"+");
        test_parsing(b"(\\53)", b"+");
        assert_err::<PdfString>(b"");
        assert_err::<PdfString>(b"(");
        assert_err::<PdfString>(b"(\\a)");
    }

    #[test]
    fn test_hex_strings() {
        test_parsing(b"<48656C6C6F20776F726C64>", b"Hello world");
    }
}