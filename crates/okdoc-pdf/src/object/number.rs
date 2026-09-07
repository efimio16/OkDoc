use std::io::BufRead;

use crate::{error::PdfError, parser::Parseable};

pub enum PdfNumber {
    /// `123`, `43445`, `+17`, `−98`, `0`
    Integer(i32),
    /// `34.5`, `−3.62`, `+123.6`, `4.`, `−.002`, `0.0`
    Real(f32),
}

impl Parseable for PdfNumber {
    fn matches_from_start(bytes: &[u8]) -> usize {
        bytes.len()
    }

    fn from_bytes<T: BufRead>(input: T) -> Result<Self, PdfError> {
        todo!()
    }
}