use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    PointOutbound,
    FontNotFound,
    Unknown(#[from] Box<dyn std::error::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
