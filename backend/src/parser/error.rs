use thiserror::Error;

/// Errors that can occur during profile parsing
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Invalid profile format: {0}")]
    InvalidFormat(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Failed to parse value: {0}")]
    ParseValue(String),
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

