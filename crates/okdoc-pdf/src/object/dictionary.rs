use std::collections::HashMap;

use super::{PdfObject, name::PdfName};

// ## Syntax
/// ```txt
/// <<
/// /Type /Example
/// /Subtype /DictionaryExample
/// /Version 0.01
/// /IntegerItem 12
/// /StringItem (a string)
/// /Subdictionary <<
///     	/Item1 0.4
/// 		/Item2 true
/// 		/LastItem (not!)
/// 		/VeryLastItem (OK)
/// 		\>>
/// \>>
/// ```
pub struct PdfDictionary(pub HashMap<PdfName, PdfObject>);
