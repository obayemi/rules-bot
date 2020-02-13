use std::fmt;
use diesel::result;

#[derive(Debug)]
pub enum BotError {
    DBError(result::Error),
}

pub struct DBError(result::Error);
impl Into<BotError> for DBError {
    fn into(self) -> BotError {
        BotError::DBError(self.0)
    }
} 

// We need this to performs a conversion from diesel::result::Error to BotError
impl From<result::Error> for BotError {
    fn from(error: result::Error) -> Self {
        BotError::DBError(error)
    }
}

// We need this so we can use the method to_string over BotError 
impl fmt::Display for BotError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BotError::DBError(error) => write!(f, "{}", error),
        }
    }
}