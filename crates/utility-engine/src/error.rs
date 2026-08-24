use std::fmt;

#[derive(Debug)]
pub struct UtilityError {
    message: String,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl UtilityError {
    #[must_use]
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
        }
    }

    #[must_use]
    pub fn with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl fmt::Display for UtilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for UtilityError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn std::error::Error + 'static))
    }
}

impl From<String> for UtilityError {
    fn from(message: String) -> Self {
        Self::message(message)
    }
}

impl From<&str> for UtilityError {
    fn from(message: &str) -> Self {
        Self::message(message)
    }
}

pub type UtilityResult<T> = Result<T, UtilityError>;
