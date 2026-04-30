use std::{fs::File, io::{BufRead, BufReader}};

pub mod error;

pub use error::ParseError;

pub struct Parser;

impl Parser {
    /// Skips header and keeps in buf the first line that doesnt start with `sym`
    pub fn skip_header(reader: &mut BufReader<File>, buf: &mut String, sym: char) -> Result<usize, ParseError> {
        loop {
            buf.clear();

            let nbytes = reader
                .read_line(buf)
                .map_err(|e| ParseError::Io(e))?;

            if nbytes == 0 {
                return Err(ParseError::EmptyBody);
            }

            let line = buf.trim();

            if line.is_empty() {
                continue;
            }

            if !line.starts_with(sym){
                return Ok(nbytes);
            }
        }
    }

}
