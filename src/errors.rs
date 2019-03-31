use failure::{
    self,
    Fail,
    Error
};

#[derive(Debug, Fail)]
pub enum SpiderError {

    #[fail(display="")]
    HatchError(#[cause] reqwest::Error),

    #[fail(display="")]
    HttpError,

    #[fail(display="")]
    HttpParseError,

    #[fail(display="")]
    FileSystemError
}

impl From<reqwest::Error> for SpiderError {
    fn from(error: reqwest::Error) -> Self {
        SpiderError::HatchError(error)
    }
}