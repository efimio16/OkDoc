use std::io::{self, BufRead};
use crate::error::PdfError;


/// Adds ability to decode objects from bytes.
/// It consists of 2 methods: `matches` and `parse_from`
pub trait Parseable
where Self: Sized {
    /// Returns how many bytes from the beginning of the slice matches with object's structure.
    fn matches(bytes: &[u8]) -> usize;
    /// Parses the object itself from bytes.
    fn parse_from<T: BufRead>(input: T) -> Result<Self, PdfError>;
}

pub trait PdfInput {
    fn read_keyword(&mut self, kw: &str) -> Result<(), PdfError>;
}

impl<T: BufRead> PdfInput for T {
    fn read_keyword(&mut self, kw: &str) -> Result<(), PdfError> {
        let mut i = 0;
        let kw_len = kw.len();
        
        loop {
            let chunk = self.fill_buf()?;
            let chunk_len = chunk.len();
            if chunk_len == 0 {
                return Err(PdfError::Io(io::ErrorKind::UnexpectedEof.into()));
            }
            
            let amt = chunk_len.min(kw_len - i);

            if &chunk[..amt] != kw[i..i+amt].as_bytes() {
                // Example: "expected `tru` in keyword `true`, but got `tro`"
                return Err(PdfError::Parse(format!(
                    "expected `{}` in keyword `{}`, but got `{}{}`",
                    &kw[..i+amt],
                    kw,
                    &kw[..i],
                    String::from_utf8_lossy(&chunk[..amt]),
                )));
            }

            self.consume(amt);
            i += amt;

            if i == kw_len {
                return Ok(());
            }
        }
    }
}

#[cfg(test)]
pub mod test_utils {
    use std::{assert_matches, fmt::Debug};
    use super::*;

    pub fn assert_slice_and_value<T: Parseable + Debug + PartialEq>(slice: &[u8], value: T) {
        assert_eq!(T::parse_from(slice).unwrap(), value);
    }

    pub fn assert_err<T: Parseable + Debug>(slice: &[u8]) {
        assert_matches!(T::parse_from(slice), Err(_));
    }
}