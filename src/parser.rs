use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

pub mod error;

pub use error::ParseError;

#[derive(Debug, Clone)]
pub struct GraphMTX {
    pub edges: Vec<(usize, usize)>,
    pub nrows: usize,
    pub ncols: usize,
    pub nnz: usize,
}

impl GraphMTX {
    pub fn new(edges: Vec<(usize, usize)>, nrows: usize, ncols: usize, nnz: usize) -> Self {
        Self {
            edges,
            nrows,
            ncols,
            nnz,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GraphTSV {
    pub ids: HashMap<String, usize>,
    pub nodes: Vec<String>,
    pub edges: Vec<(usize, usize)>,
    pub categories: Vec<Vec<Vec<String>>>,
}

impl GraphTSV {
    pub fn new(
        ids: HashMap<String, usize>,
        nodes: Vec<String>,
        edges: Vec<(usize, usize)>,
        categories: Vec<Vec<Vec<String>>>,
    ) -> Self {
        Self {
            ids,
            nodes,
            edges,
            categories,
        }
    }
}

pub struct Parser;

impl Parser {
    /// Skips header and keeps in buf the first line that doesnt start with `sym`
    pub fn skip_header(
        reader: &mut BufReader<File>,
        buf: &mut String,
        sym: char,
    ) -> Result<usize, ParseError> {
        loop {
            buf.clear();

            let nbytes = reader.read_line(buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                return Err(ParseError::EmptyBody);
            }

            let line = buf.trim();

            if line.is_empty() {
                continue;
            }

            if !line.starts_with(sym) {
                return Ok(nbytes);
            }
        }
    }

    pub fn parse_tsv(
        path_articles: &str,
        path_categories: &str,
        path_links: &str,
    ) -> Result<GraphTSV, ParseError> {
        let mut buf = String::new();

        // parse articles
        let mut ids = HashMap::new();
        let mut nodes = Vec::new();

        let file_articles = File::open(path_articles).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_articles);

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            ids.insert(line.to_string(), nodes.len());
            nodes.push(line.to_string());

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        // parse categories
        let mut categories = vec![vec![]; nodes.len()];

        let file_categories = File::open(path_categories).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_categories);

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            let (name, category) = line
                .split_once('\t')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            let name_id = ids.get(name).ok_or(ParseError::Inconsistent {
                reason: format!("name {name} not found in nodes"),
                line: buf.clone(),
            })?;

            categories[*name_id].push(category.split('.').skip(1).map(|s| s.to_string()).collect());

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        // parse edges
        let file_links = File::open(path_links).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file_links);

        let mut edges = Vec::new();

        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;

        loop {
            let line = buf.trim();

            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }

            let (src, dst) = line
                .split_once('\t')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            let src_id = ids.get(src).ok_or(ParseError::Inconsistent {
                reason: format!("name {src} not found in nodes"),
                line: buf.clone(),
            })?;

            let dst_id = ids.get(dst).ok_or(ParseError::Inconsistent {
                reason: format!("name {dst} not found in nodes"),
                line: buf.clone(),
            })?;

            edges.push((*src_id, *dst_id));

            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                break;
            }
        }

        Ok(GraphTSV::new(ids, nodes, edges, categories))
    }

    pub fn parse_mtx(path: &str) -> Result<GraphMTX, ParseError> {
        let file = File::open(path).map_err(ParseError::Io)?;

        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        // skip header
        let _ = Parser::skip_header(&mut reader, &mut buf, '%')?;

        let mut split = buf.split_whitespace();

        let nrows = split
            .next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let ncols = split
            .next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let nnz = split
            .next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let mut edges = Vec::with_capacity(nnz);

        // parse edges
        for i in 0..nnz {
            buf.clear();

            let nbytes = reader.read_line(&mut buf).map_err(ParseError::Io)?;

            if nbytes == 0 {
                return Err(ParseError::TooShort {
                    expected: nnz,
                    got: i,
                });
            }

            let (src, dst) = buf
                .trim()
                .split_once(' ')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            edges.push((
                src.parse::<usize>()
                    .map_err(|_| ParseError::BadLine(buf.clone()))?
                    - 1,
                dst.parse::<usize>()
                    .map_err(|_| ParseError::BadLine(buf.clone()))?
                    - 1,
            ));
        }

        Ok(GraphMTX::new(edges, nrows, ncols, nnz))
    }
}
