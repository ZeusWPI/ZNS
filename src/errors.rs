use core::fmt;

use crate::structs::RCODE;

pub struct DNSError {
    pub message: String,
    pub rcode: RCODE,
}

impl fmt::Display for DNSError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub object: String,
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse Error for {}: {}", self.object, self.message)
    }
}

#[derive(Debug)]
pub struct DatabaseError {
    pub message: String,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Database Error: {}", self.message)
    }
}

#[derive(Debug)]
pub struct AuthenticationError {
    pub message: String,
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Authentication Error: {}", self.message)
    }
}

#[derive(Debug)]
pub struct ReaderError {
    pub message: String,
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Reader Error: {}", self.message)
    }
}

impl<E> From<E> for ParseError
where
    E: Into<ReaderError>,
{
    fn from(value: E) -> Self {
        ParseError {
            object: String::from("Reader"),
            message: value.into().to_string(),
        }
    }
}

impl<E> From<E> for DNSError
where
    E: Into<ParseError>,
{
    fn from(value: E) -> Self {
        DNSError {
            message: value.into().to_string(),
            rcode: RCODE::FORMERR,
        }
    }
}

trait Supported {}

impl Supported for reqwest::Error {}
impl Supported for DatabaseError {}

impl<E> From<E> for AuthenticationError
where
    E: Supported,
    E: std::fmt::Display,
{
    fn from(value: E) -> Self {
        AuthenticationError {
            message: value.to_string(),
        }
    }
}
