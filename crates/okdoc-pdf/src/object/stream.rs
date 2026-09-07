use bytes::Bytes;

/// ## Syntax
/// ```txt
/// stream
/// ...bytes...
/// endstream
/// ```
pub struct PdfStream(pub Bytes);