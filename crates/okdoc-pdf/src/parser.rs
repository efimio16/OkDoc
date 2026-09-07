use std::io::BufRead;

use crate::error::PdfError;

pub trait Parseable
where Self: Sized {
    /// Requires at least 20 bytes.
    /// Returns how many bytes from the start of the slice really matches.
    fn matches_from_start(bytes: &[u8]) -> usize;
    /// Reads bytes, parses them and finally returns the object itself.
    fn from_bytes<T: BufRead>(input: T) -> Result<Self, PdfError>;
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
