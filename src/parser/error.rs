#[derive(Debug)]
pub enum ParseError {
    Io(std::io::Error),
    BadLine(String),
    EmptyBody,
    TooShort {expected: usize, got: usize},
    Inconsistent {reason: String, line: String},
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(error) => writeln!(f, "{error}"),
            ParseError::BadLine(line) => writeln!(f, "failed to parse line: {line}"),
            ParseError::EmptyBody => writeln!(f, "cannot parse a file without body"),
            ParseError::TooShort { expected, got } => writeln!(f, "too little edges, expected: {expected}, got: {got}"),
            ParseError::Inconsistent { reason, line } => writeln!(f, "{reason}: {line}"),
        }
    }
}

impl std::error::Error for ParseError {}
