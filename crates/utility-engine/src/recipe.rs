use crate::{DataValue, UtilityError, UtilityResult};
use serde_json::Value;

pub const MAX_BATCH_ITEMS: usize = 100;
pub const MAX_RECIPE_STEPS: usize = 64;

#[derive(Clone, Debug)]
pub struct RecipeStep {
    pub operation: String,
    pub args: Value,
}

pub fn run_recipe(
    mut value: DataValue,
    steps: &[RecipeStep],
    mut execute: impl FnMut(&str, DataValue, &Value) -> UtilityResult<DataValue>,
) -> UtilityResult<DataValue> {
    if steps.len() > MAX_RECIPE_STEPS {
        return Err(UtilityError::message(format!(
            "recipe must contain at most {MAX_RECIPE_STEPS} steps"
        )));
    }
    value.ensure_bounded("input")?;
    for step in steps {
        value = execute(&step.operation, value, &step.args)?;
        value.ensure_bounded("output")?;
    }
    Ok(value)
}
