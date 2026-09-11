use std::io::{BufRead, ErrorKind::UnexpectedEof};
use crate::{error::PdfError, parseable::Parseable};

/// ## Syntax
/// `1 0 R`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PdfReference {
    pub number: u32,
    pub generation: u16,
}

impl PdfReference {
    pub fn new(number: u32, generation: u16) -> Self {
        Self { number, generation }
    } 
}

impl Parseable for PdfReference {
    fn matches(_: &[u8]) -> usize {
        todo!()
    }
    fn parse_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        #[derive(Default)]
        enum State {
            #[default]
            Number,
            Generation,
            Keyword,
        }

        let mut state = State::default();
        let mut value = Self::default();

        loop {
            let chunk = input.fill_buf()?;
            if chunk.len() == 0 {
                return Err(PdfError::Io(UnexpectedEof.into()));
            }

            let mut i = 0;

            for b in chunk {
                match state {
                    State::Number => {
                        match b {
                            b' ' => state = State::Generation,
                            b'0'..=b'9' => {
                                value.number = if let Some(v) = value.number
                                    .checked_mul(10)
                                    .map(|v| v.checked_add((b - b'0') as u32))
                                    .flatten()
                                {
                                    v
                                } else {
                                    return Err(PdfError::Parse("overflow occured".into()))
                                };
                            }
                            _ => return Err(PdfError::Parse(format!("invalid character: {}", *b as char))),
                        }
                    }
                    State::Generation => {
                        match b {
                            b' ' => state = State::Keyword,
                            b'0'..=b'9' => {
                                value.generation = if let Some(v) = value.generation
                                    .checked_mul(10)
                                    .map(|v| v.checked_add((b - b'0') as u16))
                                    .flatten()
                                {
                                    v
                                } else {
                                    return Err(PdfError::Parse("overflow occured".into()))
                                };
                            }
                            _ => return Err(PdfError::Parse(format!("invalid character: {}", *b as char))),
                        }
                    }
                    State::Keyword => {
                        match b {
                            b'R' => {
                                input.consume(i + 1);
                                return Ok(value);
                            }
                            _ => return Err(PdfError::Parse(format!("invalid character: {}", *b as char))),
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
    fn test_pdf_references() {
        assert_parsing(b"1 0 R", PdfReference::new(1, 0), b"");
        assert_parsing(b"100 200 R something else", PdfReference::new(100, 200), b" something else");
        
        assert_err::<PdfReference>(b"9999999999 99999 R "); // overflow
        assert_err::<PdfReference>(b"1 2  R ");             // bad spacing
    }
}