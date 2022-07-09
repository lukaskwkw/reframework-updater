use std::error::Error;

use std::fmt::Display;

#[derive(Debug)]
pub enum UnzipError {
    OutpathFileName,
    other,
    zip_new_err,
    by_index_err,
    create_dir_all,
    file_create,
    io_copy,
}

impl Display for UnzipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("Local error")
    }
}

impl Error for UnzipError {}
