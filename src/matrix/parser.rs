use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::parser::{Parser, ParseError};

use faer::sparse::{SparseColMat, Triplet};

pub struct MatrixParser;

impl MatrixParser {
    pub fn parse_tsv(
        path_articles: &str, 
        path_links: &str
    ) -> Result<SparseColMat<usize, f64>, ParseError> 
    {
        let mut buf = String::new();
    
        // count the number of nodes 
        let mut ids = HashMap::new();
        let mut nodes = Vec::new();
        
        let file_articles = File::open(path_articles)
            .map_err(|e| ParseError::Io(e))?;
    
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
    
            let nbytes = reader
                .read_line(&mut buf)
                .map_err(|e| ParseError::Io(e))?;
    
            if nbytes == 0 {
                break;
            }
        }
    
        // parse edges
        let file_links = File::open(path_links)
            .map_err(|e| ParseError::Io(e))?;
    
        let mut reader = BufReader::new(file_links);
    
        let mut triplets = Vec::new();
    
        let _ = Parser::skip_header(&mut reader, &mut buf, '#')?;
    
        loop {
            let line = buf.trim();
    
            if line.is_empty() {
                return Err(ParseError::BadLine(buf));
            }
    
            let (src, dst) = line
                .split_once('\t')
                .ok_or(ParseError::BadLine(buf.clone()))?;
    
            let src_id = ids
                .get(src)
                .ok_or(ParseError::Inconsistent {
                    reason: format!("name {src} not found in nodes"),
                    line: buf.clone(),
                })?;
    
            let dst_id = ids
                .get(dst)
                .ok_or(ParseError::Inconsistent {
                    reason: format!("name {dst} not found in nodes"),
                    line: buf.clone(),
                })?;
    
            triplets.push(Triplet::new(*dst_id, *src_id, 1.0));
    
            buf.clear();
    
            let nbytes = reader
                .read_line(&mut buf)
                .map_err(|e| ParseError::Io(e))?;
    
            if nbytes == 0 {
                break;
            }
        }

        let n_nodes = nodes.len();
        let mat = SparseColMat::try_new_from_triplets(n_nodes, n_nodes, &triplets)
            .map_err(|err| ParseError::MatrixError(err))?;

        Ok(mat)
    }
    
    
    pub fn parse_mtx(path: &str) -> Result<SparseColMat<usize, f64>, ParseError> {
        let file = File::open(path)
            .map_err(|e| ParseError::Io(e))?;

        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        // skip header
        let _ = Parser::skip_header(&mut reader, &mut buf, '%')?;

        let mut split = buf.split_whitespace();

        let nrows = split.next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let ncols = split.next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let nnz = split.next()
            .ok_or(ParseError::BadLine(buf.clone()))?
            .parse()
            .map_err(|_| ParseError::BadLine(buf.clone()))?;

        let mut triplets = Vec::with_capacity(nnz);

        // parse edges
        for i in 0..nnz {
            buf.clear();

            let nbytes = reader.read_line(&mut buf)
                .map_err(|e| ParseError::Io(e))?;

            if nbytes == 0 {
                return Err(ParseError::TooShort {
                    expected: nnz,
                    got: i
                });
            }

            let (src, dst) = buf.trim().split_once(' ')
                .ok_or(ParseError::BadLine(buf.clone()))?;

            triplets.push(
                Triplet::new(
                    dst.parse::<usize>().map_err(|_| ParseError::BadLine(buf.clone()))? - 1,
                    src.parse::<usize>().map_err(|_| ParseError::BadLine(buf.clone()))? - 1,
                    1.0
                )
            );
        }

        let mat = SparseColMat::try_new_from_triplets(nrows, ncols, &triplets)
            .map_err(|err| ParseError::MatrixError(err))?;

        Ok(mat)
    }
}
