use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const MAX_UTILITY_INPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_BATCH_ITEMS: usize = 100;
pub const MAX_RECIPE_STEPS: usize = 64;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DataValue {
    Text(String),
    Bytes(Vec<u8>),
    Json(Value),
}

impl DataValue {
    pub fn encoded_len(&self) -> Result<usize, String> {
        match self {
            Self::Text(value) => Ok(value.len()),
            Self::Bytes(value) => Ok(value.len()),
            Self::Json(value) => serde_json::to_vec(value)
                .map(|encoded| encoded.len())
                .map_err(|error| error.to_string()),
        }
    }

    pub fn ensure_bounded(&self, label: &str) -> Result<(), String> {
        let size = self.encoded_len()?;
        if size > MAX_UTILITY_INPUT_BYTES {
            Err(format!("{label} exceeds {MAX_UTILITY_INPUT_BYTES} bytes"))
        } else {
            Ok(())
        }
    }

    pub fn as_bytes(&self) -> Result<&[u8], String> {
        match self {
            Self::Text(value) => Ok(value.as_bytes()),
            Self::Bytes(value) => Ok(value),
            Self::Json(_) => Err("operation does not accept JSON input".to_owned()),
        }
    }

    pub fn as_text(&self) -> Result<&str, String> {
        match self {
            Self::Text(value) => Ok(value),
            _ => Err("operation requires text input".to_owned()),
        }
    }

    pub fn parse_json(&self) -> Result<Value, String> {
        match self {
            Self::Json(value) => Ok(value.clone()),
            Self::Text(value) => serde_json::from_str(value).map_err(|error| error.to_string()),
            Self::Bytes(_) => Err("JSON operation requires text or JSON input".to_owned()),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct OperationInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub input_kind: &'static str,
    pub output_kind: &'static str,
    pub deterministic: bool,
    pub pure: bool,
    pub cryptographically_weak: bool,
}

#[derive(Clone, Debug)]
pub struct RecipeStep {
    pub operation: String,
    pub args: Value,
}

pub fn run_recipe(
    mut value: DataValue,
    steps: &[RecipeStep],
    mut execute: impl FnMut(&str, DataValue, &Value) -> Result<DataValue, String>,
) -> Result<DataValue, String> {
    if steps.len() > MAX_RECIPE_STEPS {
        return Err(format!(
            "recipe must contain at most {MAX_RECIPE_STEPS} steps"
        ));
    }
    value.ensure_bounded("input")?;
    for step in steps {
        value = execute(&step.operation, value, &step.args)?;
        value.ensure_bounded("output")?;
    }
    Ok(value)
}
