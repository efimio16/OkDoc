fn main() -> std::io::Result<()> {
    let document = std::fs::read("sample-100kb.pdf")?;
    
    // Access PDF version
    println!("PDF: {}", String::from_utf8_lossy(&document));
    
    // // Get cross-reference table
    // let xrefs = document.get_xref_slice();
    // println!("XRef entries: {}", xrefs.len());

    Ok(())
}