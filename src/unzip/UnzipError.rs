use std::error::Error;

use std::fmt::Display;

#[derive(Debug)]
pub enum UnzipError {
    OutpathFileName,
}

impl Display for UnzipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UnzipError::*;
        match self {
            UnzipError::OutpathFileName => write!(f, "{}", OutpathFileName),
        }
    }
}

impl Error for UnzipError {}
