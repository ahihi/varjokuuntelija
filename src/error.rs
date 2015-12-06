use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct CustomError {
    pub message: String
}

impl CustomError {
    pub fn new(message: &str) -> CustomError {
        CustomError { message: message.to_string() }
    } 
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CustomError {
    fn description(&self) -> &str {
        &self.message
    }
    
    fn cause(&self) -> Option<&Error> {
        None
    }
}
