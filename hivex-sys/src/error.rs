//
use std::{error, fmt, num, path};

#[derive(Debug)]
pub enum ErrorKind {
    HiveFile { path: path::PathBuf },
    Ffi { description: String },
    Hivex { function: String },
    StrConvert,
    IntConvert,
}

// Error
#[derive(Debug)]
pub struct Error {
    reason: String,
    kind: ErrorKind,
}

impl From<num::TryFromIntError> for Error {
    fn from(_err: num::TryFromIntError) -> Self {
        Self::int_convert_error()
    }
}

impl Error {
    pub fn hive_does_not_exist<P>(path: P) -> Self
    where
        P: AsRef<path::Path>,
    {
        Self {
            reason: "The path provided does not exist".into(),
            kind: ErrorKind::HiveFile {
                path: path.as_ref().to_path_buf(),
            },
        }
    }
    pub fn string_convert_error() -> Self {
        Self {
            reason: "Could not convert to str".into(),
            kind: ErrorKind::StrConvert,
        }
    }
    pub fn int_convert_error() -> Self {
        Self {
            reason: "Could not convert int".into(),
            kind: ErrorKind::IntConvert,
        }
    }
    pub fn ffi_error<E>(err: E) -> Self
    where
        E: error::Error,
    {
        Self {
            reason: "An FFI error occurred".into(),
            kind: ErrorKind::Ffi {
                description: format!("{}", err.to_string()),
            },
        }
    }
    pub fn hivex_error(function: &str) -> Self {
        Self {
            reason: format!("Hivex function {} encountered an error", function),
            kind: ErrorKind::Hivex {
                function: function.to_string(),
            },
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::HiveFile { path: _ } => {
                write!(fmt, "File Error, {}: {:?}", self.reason, self.kind)
            }
            ErrorKind::Ffi { description: _ } => write!(fmt, "FFI Error, {}", self.reason),
            ErrorKind::Hivex { function: _ } => {
                write!(fmt, "Hivex Error, {}: {:?}", self.reason, self.kind)
            }
            ErrorKind::StrConvert => write!(fmt, "Str Error, {}", self.reason),
            ErrorKind::IntConvert => write!(fmt, "Int Error, {}", self.reason),
        }
    }
}

impl error::Error for Error {}
