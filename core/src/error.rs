use std::{
    error, fmt, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Error {
    path: PathBuf,
    err: io::Error,
}

impl Error {
    pub(crate) fn new(path: impl Into<PathBuf>, err: io::Error) -> Self {
        Error {
            path: path.into(),
            err,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn io_error(&self) -> &io::Error {
        &self.err
    }

    pub fn kind(&self) -> io::ErrorKind {
        self.err.kind()
    }

    pub fn into_io_error(self) -> io::Error {
        self.err
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.err)
    }
}

// No source(): the io error is already part of the message.
impl error::Error for Error {}

// Keeps the path in the message; into_io_error() keeps the OS error code instead.
impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(err.kind(), err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    // 2 is "not found" on both Unix (ENOENT) and Windows (ERROR_FILE_NOT_FOUND).
    fn not_found() -> Error {
        Error::new("some/dir", io::Error::from_raw_os_error(2))
    }

    #[test]
    fn test_error_keeps_path_and_os_error() {
        let err = not_found();

        assert_eq!(err.path(), Path::new("some/dir"));
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(err.io_error().raw_os_error(), Some(2));
        assert!(err.source().is_none());
        let os_err = io::Error::from_raw_os_error(2);
        assert_eq!(err.to_string(), format!("some/dir: {}", os_err));
    }

    #[test]
    fn test_into_io_error_returns_os_error() {
        assert_eq!(not_found().into_io_error().raw_os_error(), Some(2));
    }

    #[test]
    fn test_io_error_from_error_keeps_kind_and_path() {
        let err: io::Error = not_found().into();

        assert_eq!(err.kind(), io::ErrorKind::NotFound);
        assert_eq!(err.to_string(), not_found().to_string());
    }
}
