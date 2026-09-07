use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PdfError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error("parse error: {0}")]
    Parse(String),
}