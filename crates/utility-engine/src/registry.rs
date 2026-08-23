use crate::{DataValue, UtilityError, UtilityResult};
use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueKind {
    Any,
    Text,
    Bytes,
    Json,
    TextOrJson,
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct OperationInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub input_kind: ValueKind,
    pub output_kind: ValueKind,
    pub deterministic: bool,
    pub pure: bool,
    pub cryptographically_weak: bool,
}

pub(crate) type Executor = fn(&'static str, DataValue, &Value) -> UtilityResult<DataValue>;

#[derive(Clone, Copy)]
pub(crate) struct Operation {
    pub info: OperationInfo,
    pub(crate) execute: Executor,
}

impl Operation {
    pub(crate) const fn new(info: OperationInfo, execute: Executor) -> Self {
        Self { info, execute }
    }

    pub(crate) fn execute(self, input: DataValue, args: &Value) -> UtilityResult<DataValue> {
        (self.execute)(self.info.id, input, args)
    }
}

pub(crate) fn run_from_registry(
    operations: &[Operation],
    id: &str,
    input: DataValue,
    args: &Value,
) -> UtilityResult<DataValue> {
    input.ensure_bounded("input")?;
    let operation = operations
        .iter()
        .copied()
        .find(|operation| operation.info.id == id)
        .ok_or_else(|| UtilityError::message(format!("unknown operation: {id}")))?;
    let output = operation.execute(input, args)?;
    output.ensure_bounded("output")?;
    Ok(output)
}
