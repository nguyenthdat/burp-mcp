use crate::{UtilityError, UtilityResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_UTILITY_INPUT_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    Text(String),
    Bytes(Vec<u8>),
    Json(Value),
}

impl DataValue {
    pub fn encoded_len(&self) -> UtilityResult<usize> {
        match self {
            Self::Text(value) => Ok(value.len()),
            Self::Bytes(value) => Ok(value.len()),
            Self::Json(value) => serde_json::to_vec(value)
                .map(|encoded| encoded.len())
                .map_err(|error| UtilityError::with_source("failed to encode JSON utility value", error)),
        }
    }

    pub fn ensure_bounded(&self, label: &str) -> UtilityResult<()> {
        let size = self.encoded_len()?;
        if size > MAX_UTILITY_INPUT_BYTES {
            Err(UtilityError::message(format!(
                "{label} exceeds {MAX_UTILITY_INPUT_BYTES} bytes"
            )))
        } else {
            Ok(())
        }
    }

    pub fn as_bytes(&self) -> UtilityResult<&[u8]> {
        match self {
            Self::Text(value) => Ok(value.as_bytes()),
            Self::Bytes(value) => Ok(value),
            Self::Json(_) => Err(UtilityError::message("operation does not accept JSON input")),
        }
    }

    pub fn as_text(&self) -> UtilityResult<&str> {
        match self {
            Self::Text(value) => Ok(value),
            _ => Err(UtilityError::message("operation requires text input")),
        }
    }

    pub fn parse_json(&self) -> UtilityResult<Value> {
        match self {
            Self::Json(value) => Ok(value.clone()),
            Self::Text(value) => serde_json::from_str(value)
                .map_err(|error| UtilityError::with_source("invalid JSON input", error)),
            Self::Bytes(_) => Err(UtilityError::message(
                "JSON operation requires text or JSON input",
            )),
        }
    }
}
