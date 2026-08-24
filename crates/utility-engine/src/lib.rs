mod error;
mod operations;
mod recipe;
mod registry;
mod value;

pub use error::{UtilityError, UtilityResult};
pub use operations::{MagicSuggestion, describe, magic, run, search};
pub use recipe::{MAX_BATCH_ITEMS, MAX_RECIPE_STEPS, RecipeStep, run_recipe};
pub use registry::{OperationInfo, ValueKind};
pub use value::{DataValue, MAX_UTILITY_INPUT_BYTES};
