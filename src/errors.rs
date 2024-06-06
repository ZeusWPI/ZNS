use core::fmt;

#[derive(Debug)]
pub struct DNSError {
    pub message: String,
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
