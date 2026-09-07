/// ## Syntax
/// ```txt
/// 1 0 obj
/// ...
/// endobj
/// ```
pub struct PdfIndirectObject {
    pub number: u32,
    pub generation: u16,
}
