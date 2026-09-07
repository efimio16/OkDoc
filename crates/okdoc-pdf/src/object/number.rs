use std::io::BufRead;

use crate::{error::PdfError, parser::Parseable};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum PdfNumber {
    /// `123`, `43445`, `+17`, `−98`, `0`
    Integer(i32),
    /// `34.5`, `−3.62`, `+123.6`, `4.`, `−.002`, `0.0`
    Real(f32),
}

impl PdfNumber {
    pub fn as_int(self) -> i32 {
        match self {
            Self::Integer(i) => i,
            Self::Real(r) => r as i32,
        }
    }
    pub fn as_real(self) -> f32 {
        match self {
            Self::Integer(i) => i as f32,
            Self::Real(r) => r,
        }
    }
}

impl Default for PdfNumber {
    fn default() -> Self {
        Self::Integer(0)
    }
}

impl Parseable for PdfNumber {
    fn matches_from_start(bytes: &[u8]) -> usize {
        bytes.len()
    }

    fn from_bytes<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        let mut sign = 1;
        let mut sign_set = false;
        
        let mut value = PdfNumber::default();
        let mut met_digit = false;
        
        let mut decimals = 0;
        
        loop {
            let mut i = 0;
            let chunk = input.fill_buf()?;

            if chunk.len() == 0 {
                return if met_digit {
                    Ok(value)
                } else {
                    Err(PdfError::Parse("expected at least one digit".to_string()))
                }
            }

            if !sign_set {
                match chunk[i] {
                    b'+' => {
                        i += 1;
                    }
                    b'-' => {
                        sign = -1;
                        i += 1;
                    }
                    _ => {}
                }
                sign_set = true;
            }

            for b in &chunk[i..] {
                value = match b {
                    b'.' => {
                        match value {
                            Self::Real(_) => {
                                return Err(PdfError::Parse("duplicated dot".to_string()));
                            }
                            Self::Integer(i) => {
                                Self::Real(i as f32)
                            }
                        }
                    }
                    b'0'..=b'9' => {
                        if !met_digit {
                            met_digit = true;
                        }
                        match value {
                            Self::Real(r) => {
                                decimals += 1;
                                Self::Real(r + (b - b'0') as f32 * 10.0f32.powi(-decimals) * sign as f32)
                            }
                            Self::Integer(i) => {
                                Self::Integer(i * 10 + (b - b'0') as i32 * sign)
                            }
                        }
                    }
                    _ => {
                        if met_digit {
                            input.consume(i);
                            return Ok(value);
                        } else {
                            return Err(PdfError::Parse("expected at least one digit".to_string()))
                        }
                    }
                };
                i += 1;
            }
            input.consume(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;
    use super::*;

    fn assert_slice_and_value(slice: &[u8], value: PdfNumber) {
        assert_eq!(PdfNumber::from_bytes(slice).unwrap(), value);
    }

    fn assert_err(slice: &[u8]) {
        assert_matches!(PdfNumber::from_bytes(slice), Err(_));
    }

    #[test]
    fn test_number() {
        assert_slice_and_value(b"100", PdfNumber::Integer(100));            // A normal integer
        assert_slice_and_value(b"+34567", PdfNumber::Integer(34567));       // Explicit positive sign
        assert_slice_and_value(b"-7", PdfNumber::Integer(-7));              // Explicit negative sign

        assert_slice_and_value(b"-.1", PdfNumber::Real(-0.1));              // Leading point
        assert_slice_and_value(b"+1111.1111", PdfNumber::Real(1111.1111));  // Embedded point
        assert_slice_and_value(b"100.", PdfNumber::Real(100.));             // Trailing point

        assert_err(b"-");
        assert_err(b"+");
        assert_err(b".");
        assert_err(b"+.");
        assert_err(b"-.");
        assert_err(b"-0..1");
        assert_err(b"1.0.3");
    }
}