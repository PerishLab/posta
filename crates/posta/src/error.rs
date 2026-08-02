#[derive(Debug)]
pub enum Error {
    Invalid(String),
    Conflict(String),
    Engine(keel::adapt::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(note) => write!(out, "invalid: {note}"),
            Self::Conflict(note) => write!(out, "conflict: {note}"),
            Self::Engine(error) => write!(out, "{error}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Engine(error) => Some(error),
            Self::Invalid(_) | Self::Conflict(_) => None,
        }
    }
}

impl From<keel::adapt::Error> for Error {
    fn from(error: keel::adapt::Error) -> Self {
        match error {
            keel::adapt::Error::Adapt(note) => classify(note),
            other => Self::Engine(other),
        }
    }
}

fn classify(note: String) -> Error {
    if let Some(note) = note.strip_prefix("posta invalid: ") {
        return Error::Invalid(note.to_string());
    }
    if let Some(note) = note.strip_prefix("posta conflict: ") {
        return Error::Conflict(note.to_string());
    }
    Error::Engine(keel::adapt::Error::Adapt(note))
}

pub(crate) fn invalid(note: &str) -> keel::adapt::Error {
    keel::adapt::Error::Adapt(format!("posta invalid: {note}"))
}

pub(crate) fn conflict(note: &str) -> keel::adapt::Error {
    keel::adapt::Error::Adapt(format!("posta conflict: {note}"))
}

pub(crate) fn engine(note: &str) -> keel::adapt::Error {
    keel::adapt::Error::Adapt(format!("posta: {note}"))
}
