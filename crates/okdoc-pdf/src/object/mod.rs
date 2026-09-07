pub mod array;
pub mod boolean;
pub mod dictionary;
pub mod indirect;
pub mod name;
pub mod number;
pub mod reference;
pub mod stream;
pub mod string;

use array::PdfArray;
use boolean::PdfBoolean;
use dictionary::PdfDictionary;
use indirect::PdfIndirectObject;
use name::PdfName;
use number::PdfNumber;
use reference::PdfReference;
use string::PdfString;
use stream::PdfStream;

pub enum PdfObject {
    Array(PdfArray),
    Boolean(PdfBoolean),
    Dictionary(PdfDictionary),
    Indirect(PdfIndirectObject),
    Name(PdfName),
    Number(PdfNumber),
    Reference(PdfReference),
    String(PdfString),
    Stream(PdfStream),
}
