use std::{fs::File, io::{BufRead, BufReader}};

#[derive(Debug, Clone, Copy)]
pub struct MtxMetadata {
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
}

impl MtxMetadata {
    pub fn new(nrows: usize, ncols: usize, nnz: usize) -> Self {
        Self {
            nrows,
            ncols,
            nnz,
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    BadLine(String),
    EmptyBody,
    TooShort {expected: usize, got: usize},
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(error) => writeln!(f, "{error}"),
            ParseError::BadLine(line) => writeln!(f, "failed to parse line: {line}"),
            ParseError::EmptyBody => writeln!(f, "cannot parse a file without body"),
            ParseError::TooShort { expected, got } => writeln!(f, "too little edges, expected: {expected}, got: {got}"),
        }
    }
}

impl std::error::Error for ParseError { }

pub struct Parser;

impl Parser {
    pub fn parse_mtx(path: &str) -> Result<(MtxMetadata, Vec<(usize, usize)>), ParseError> {
        let file = File::open(path)
            .map_err(|e| ParseError::Io(e))?;

        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        // skip header
        loop {
            buf.clear();

            let nbytes = reader
                .read_line(&mut buf)
                .map_err(|e| ParseError::Io(e))?;

            if nbytes == 0 {
                return Err(ParseError::EmptyBody);
            }

            let line = buf.trim();

            if line.is_empty() {
                continue;
            }

            if !line.starts_with('%'){
                break;
            }
        }

        let mut split = buf.split_whitespace();

        // parse metadata
        let metadata = MtxMetadata::new(
            split.next()
                .ok_or(ParseError::BadLine(buf.clone()))?
                .parse()
                .map_err(|_| ParseError::BadLine(buf.clone()))?,
            split.next()
                .ok_or(ParseError::BadLine(buf.clone()))?
                .parse()
                .map_err(|_| ParseError::BadLine(buf.clone()))?,
            split.next()
                .ok_or(ParseError::BadLine(buf.clone()))?
                .parse()
                .map_err(|_| ParseError::BadLine(buf.clone()))?,
        );

        let mut edges = Vec::with_capacity(metadata.nnz);

        // parse edges
        for i in 0..metadata.nnz {
            buf.clear();

            let nbytes = reader.read_line(&mut buf)
                .map_err(|e| ParseError::Io(e))?;

            if nbytes == 0 {
                return Err(ParseError::TooShort {
                    expected: metadata.nnz,
                    got: i
                });
            }

            let (src, dst) = buf.trim().split_once(' ')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            edges.push((
                src.parse::<usize>().map_err(|_| ParseError::BadLine(buf.clone()))? - 1,
                dst.parse::<usize>().map_err(|_| ParseError::BadLine(buf.clone()))? - 1,
            ));
        }

        Ok((metadata, edges))
    }

}
