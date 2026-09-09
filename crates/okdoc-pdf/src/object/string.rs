use std::io::{BufRead, ErrorKind};

use bytes::{BufMut, Bytes, BytesMut};
use faster_hex::hex_decode;
use memchr::{memchr_iter, memchr3_iter};

use crate::{error::PdfError, parseable::Parseable};

/// A string object in PDF.
/// 
/// It's not using [`String`], because Rust strings depend on UTF-8, while PDF
/// can have different character encodings.
/// 
/// ## Syntax
/// Literals: `(Strings may contain balanced parentheses ( ) and special characters (*!&}^% and so on).)`
/// 
/// Hexadecimals: `<4E6F762073686D6F7A206B6120706F702E>`
pub struct PdfString(pub Bytes);

impl PdfString {
    fn parse_literal_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        #[derive(Default)]
        enum State {
            #[default]
            CopyBytes,
            EscapeChar,
            EscapeSequence { digits_count: u8, sequence: u16 },
            EscapeMaybeEndOfLine,
        }
        
        let mut state = State::default();
        let mut value = BytesMut::new();
        
        // Allows balanced pairs of parentheses (e.g. `(Strings may contain balanced parentheses (like these).)` )
        let mut parentheses_depth = 0;

        loop {
            let chunk = input.fill_buf()?;
            if chunk.len() == 0 {
                return Err(PdfError::Io(ErrorKind::UnexpectedEof.into()));
            }
            
            let mut i = 0;
            while i < chunk.len() {
                match &mut state {
                    State::CopyBytes => {
                        while let Some(j) = memchr3_iter(b'(', b')', b'\\', &chunk[i..]).next() {
                            value.put(&chunk[i..i+j]);
                            i += j + 1;
                            match chunk[i+j] {
                                b'(' => parentheses_depth += 1,
                                b')' => {
                                    if parentheses_depth == 0 {
                                        input.consume(i);
                                        return Ok(Self(value.freeze()))
                                    }
                                    parentheses_depth -= 1;
                                }
                                b'\\' => {
                                    state = State::EscapeChar;
                                }
                                _ => unreachable!(),
                            }
                        }
                        value.put(&chunk[i..]);
                        i = chunk.len();
                    }
                    State::EscapeChar => {
                        let b = chunk[i];
                        match b {
                            b'n' => value.put_u8(b'\n'),
                            b'r' => value.put_u8(b'\r'),
                            b't' => value.put_u8(b'\t'),
                            b'b' => value.put_u8(0x8),
                            b'f' => value.put_u8(0xc),
                            b'(' | b')' | b'\\' => value.put_u8(b),
                            b'\n' | b'\r' => {
                                i += 1;
                                state = State::EscapeMaybeEndOfLine;
                                continue;
                            }
                            b'0'..=b'7' => {
                                state = State::EscapeSequence { digits_count: 0, sequence: 0 };
                                continue;
                            }
                            _ => return Err(PdfError::Parse(format!("invalid character to escape: {}", b))),
                        }
                        i += 1;
                        state = State::CopyBytes;
                    }
                    State::EscapeSequence { digits_count, sequence } => {
                        let b = chunk[i];
                        match b {
                            b'0'..=b'7' => {
                                *sequence = *sequence * 8 + (b - b'0') as u16;
                                *digits_count += 1;
                                if *digits_count == 3 {
                                    state = State::CopyBytes;
                                }
                            }
                            _ => {
                                value.put_u8((*sequence >> 8) as u8);
                                value.put_u8((*sequence & 8) as u8);
                                state = State::CopyBytes;
                                continue;
                            }
                        }
                        i += 1;
                    }
                    State::EscapeMaybeEndOfLine => {
                        match chunk[i] {
                            b'\n' | b'\r' => i += 1,
                            _ => {}
                        }
                        state = State::CopyBytes;
                    }
                }
            }

            input.consume(i);
        }
    }
    fn parse_hex_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        // macro_rules! parse_hex {
        //     () => {
                
        //     };
        // }

        let mut value = BytesMut::new();
        let mut remaining_byte = None;
        
        loop {
            let chunk = input.fill_buf()?;
            if chunk.len() == 0 {
                return Err(PdfError::Io(ErrorKind::UnexpectedEof.into()));
            }
            
            while let Some(i) = memchr_iter(b'>', chunk).next() {
                // parse hex
                input.consume(i + 1);
                return Ok(Self(value.freeze()));
            }
            // parse hex
            let mut i = 0;

            if let Some(remaining_byte) = remaining_byte {
                // let decoded_byte = hex * 16;
            }
            
            while i < chunk.len() {
                let mut out = [0u8; 1024];
                let input_len = (out.len() * 2).min(chunk.len() - i) / 2 * 2;
                
                // hex_decode(&chunk[i..i+input_len], &mut out)?;

                value.put(&out[..input_len / 2]);
                i += input_len;
                
                if chunk.len() - i == 1 {
                    remaining_byte = Some(chunk[i]);
                    i += 1;
                }
            }
            
            let chunk_len = chunk.len();
            input.consume(chunk_len);
        }
    }
}

impl Parseable for PdfString {
    fn matches(_: &[u8]) -> usize {
        todo!()
    }

    fn parse_from<T: BufRead>(mut input: T) -> Result<Self, PdfError> {
        let chunk = input.fill_buf()?;
        if chunk.len() == 0 {
            return Err(PdfError::Io(ErrorKind::UnexpectedEof.into()));
        }

        let first_byte = chunk[0];
        input.consume(1);

        match first_byte {
            // Literal string
            b'(' => Self::parse_literal_from(input),
            // WIP
            // Hex string
            b'<' => Self::parse_hex_from(input),
            _ => return Err(PdfError::Parse(format!("unexpected first character: {}", first_byte as char))),
        }
    }
}