use std::io::{self, BufRead};
use bytes::{BufMut, Bytes, BytesMut};

use crate::{error::PdfError, parseable::Parseable};

#[derive(Default)]
enum State {
    #[default]
    Slash,
    RegularChar,
    EscapeSequence { digits: u8, character: u8 },
}

/// A name object is an atomic symbol uniquely defined by a sequence of any
/// characters (8-bit values) except null (character code 0).
/// 
/// ## Syntax
/// `/Name1`
// TODO: make a byte interning
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PdfName(pub Bytes);

impl Parseable for PdfName {
    fn matches(_: &[u8]) -> usize {
        todo!("decide if this function is still needed")
    }

    fn parse_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        const MAX_NAME_LENGTH: usize = 127;
        let mut value = BytesMut::new();
        let mut state = State::default();

        loop {
            let mut i = 0;
            let chunk = input.fill_buf()?;
            if chunk.len() == 0 {
                return match state {
                    State::RegularChar => Ok(Self(value.freeze())),
                    _ => Err(PdfError::Io(io::ErrorKind::UnexpectedEof.into())),
                }
            }

            for b in chunk {
                match &mut state {
                    State::Slash => {
                        if *b != b'/' {
                            return Err(PdfError::Parse(format!("expected `/`, got {}", *b as char)));
                        }
                        state = State::RegularChar;
                    }
                    State::RegularChar => {
                        match b {
                            // PDF whitespace/delimiter characters
                            0 | 9 | 10 | 12 | 13 | 32 |
                            b'(' | b')' | b'<' | b'>' | b'[' | b']' | b'{' | b'}' | b'/' | b'%' => {
                                input.consume(i);
                                return Ok(Self(value.freeze()));
                            }
                            // Escape sequence like `#20`
                            b'#' => state = State::EscapeSequence { digits: 0, character: 0 },
                            // Regular character
                            b'!'..=b'~' => {
                                value.put_u8(*b);
                                if value.len() == MAX_NAME_LENGTH {
                                    return Err(PdfError::Parse("name too long".into()))
                                }
                            }
                            // Other
                            _ => return Err(PdfError::Parse(format!("invalid char: {} (0x{:x})", *b as char, b))),
                        }
                    }
                    State::EscapeSequence { digits, character } => {
                        *character = *character * 16 + match b {
                            b'0'..=b'9' => b - b'0',
                            b'A'..=b'F' => b - b'A' + 10,
                            b'a'..=b'f' => b - b'a' + 10,
                            _ => return Err(PdfError::Parse(format!("invalid char: expected a hex digit, got {} (0x{:x})", *b as char, b))),
                        };
                        
                        *digits += 1;
                        if *digits == 2 {
                            value.put_u8(*character);
                            if value.len() == MAX_NAME_LENGTH {
                                return Err(PdfError::Parse("name too long".into()))
                            }
                            state = State::RegularChar;
                        }
                    }
                }
                i += 1;
            }

            input.consume(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parseable::test_utils::{assert_err, assert_parsing};
    use super::*;

    #[test]
    fn test_name() {
        assert_parsing(b"/Name1", PdfName("Name1".into()), b"");
        assert_parsing(b"/ASomewhatLongerName", PdfName("ASomewhatLongerName".into()), b"");
        assert_parsing(b"/A;Name_With-Var!ous***Characters~?", PdfName("A;Name_With-Var!ous***Characters~?".into()), b"");
        assert_parsing(b"/1.2", PdfName("1.2".into()), b"");
        assert_parsing(b"/$$", PdfName("$$".into()), b"");
        assert_parsing(b"/@pattern", PdfName("@pattern".into()), b"");
        assert_parsing(b"/.notdef", PdfName(".notdef".into()), b"");
        assert_parsing(b"/", PdfName("".into()), b"");
        assert_parsing(b"/lime#20Green", PdfName("lime Green".into()), b"");
        assert_parsing(b"/paired#28#29parentheses", PdfName("paired()parentheses".into()), b"");
        assert_parsing(b"/The_Key_of_F#23_Minor", PdfName("The_Key_of_F#_Minor".into()), b"");
        assert_parsing(b"/A#42", PdfName("AB".into()), b"");
        assert_parsing(b"/Delimited%im comment", PdfName("Delimited".into()), b"%im comment");

        assert_err::<PdfName>(b"");
        assert_err::<PdfName>(b"/I'm_incompleted_#1");
        assert_err::<PdfName>(b"/That's#20NOT_##_okay");
    }
}