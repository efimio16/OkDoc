use std::io::{self, BufRead};

use crate::{error::PdfError, parser::Parseable};

#[derive(Default)]
enum State {
    #[default]
    Slash,
    RegularChar,
    EscapeSequence { digits: u8, character: u8 },
}

/// ## Syntax
/// `/Name1`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PdfName(pub String);

impl Parseable for PdfName {
    fn matches_from_start(_: &[u8]) -> usize {
        todo!("decide if this function is still needed")
    }

    fn from_bytes<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        let mut value = String::new();
        let mut state = State::default();

        loop {
            let mut amt = 0;
            let chunk = input.fill_buf()?;
            if chunk.len() == 0 {
                return match state {
                    State::RegularChar => Ok(Self(value)),
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
                            // Slash or PDF whitespace characters
                            b'/' | 0 | 9 | 10 | 12 | 13 | 32 => return Ok(Self(value)),
                            // Escape sequence like `#20`
                            b'#' => state = State::EscapeSequence { digits: 0, character: 0 },
                            // Regular character
                            b'!'..=b'~' => value.push(*b as char),
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
                            value.push(*character as char);
                            state = State::RegularChar;
                        }
                    }
                }
                amt += 1;
            }

            input.consume(amt);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::test_utils::{assert_err, assert_slice_and_value};
    use super::*;

    #[test]
    fn test_name() {
        assert_slice_and_value(b"/Name1", PdfName("Name1".into()));
        assert_slice_and_value(b"/ASomewhatLongerName", PdfName("ASomewhatLongerName".into()));
        assert_slice_and_value(b"/A;Name_With-Var!ous***Characters~?", PdfName("A;Name_With-Var!ous***Characters~?".into()));
        assert_slice_and_value(b"/1.2", PdfName("1.2".into()));
        assert_slice_and_value(b"/$$", PdfName("$$".into()));
        assert_slice_and_value(b"/@pattern", PdfName("@pattern".into()));
        assert_slice_and_value(b"/.notdef", PdfName(".notdef".into()));
        assert_slice_and_value(b"/", PdfName("".into()));
        assert_slice_and_value(b"/lime#20Green", PdfName("lime Green".into()));
        assert_slice_and_value(b"/paired#28#29parentheses", PdfName("paired()parentheses".into()));
        assert_slice_and_value(b"/The_Key_of_F#23_Minor", PdfName("The_Key_of_F#_Minor".into()));
        assert_slice_and_value(b"/A#42", PdfName("AB".into()));

        assert_err::<PdfName>(b"");
        assert_err::<PdfName>(b"/I'm_incompleted_#1");
        assert_err::<PdfName>(b"/That's#20NOT_##_okay");
    }
}