use std::{fs::File, io::{BufRead, BufReader}};

const DATA_PATH: &str = "data/web-Stanford.mtx";

#[derive(Debug, Clone, Copy)]
struct GraphMetadata {
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
}

impl GraphMetadata {
    fn new(nrows: usize, ncols: usize, nnz: usize) -> Self {
        Self {
            nrows,
            ncols,
            nnz,
        }
    }
}

#[derive(Debug)]
enum ParseError {
    Io(std::io::Error),
    BadLine(String),
    EmptyBody,
    TooShort {expected: usize, got: usize},
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(error) => writeln!(f, "Io({error})"),
            ParseError::BadLine(line) => writeln!(f, "BadLine({line})"),
            ParseError::EmptyBody => writeln!(f, "EmptyBody"),
            ParseError::TooShort { expected, got } => writeln!(f, "TooShort(expected: {expected}, got: {got})"),
        }
    }
}

impl std::error::Error for ParseError { }

fn parse_matrix_market(path: &str) -> Result<(GraphMetadata, Vec<(usize, usize)>), ParseError> {
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
    let metadata = GraphMetadata::new(
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
            src.parse().map_err(|_| ParseError::BadLine(buf.clone()))?,
            dst.parse().map_err(|_| ParseError::BadLine(buf.clone()))?,
        ));
    }

    Ok((metadata, edges))
}

fn main() {
    let (metadata, edges) = match parse_matrix_market(DATA_PATH) {
        Ok((m, e)) => (m, e),
        Err(e) => panic!("{e}"),
    };

    dbg!(metadata);
    dbg!(&edges[..10]);
}
