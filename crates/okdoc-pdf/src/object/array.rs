use super::PdfObject;

/// ## Syntax
/// `[549 3.14 false (Ralph) /SomeName]`
pub struct PdfArray(pub Vec<PdfObject>);
