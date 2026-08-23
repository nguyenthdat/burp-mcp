use super::StorageError;
use crate::limits::PageLimit;

pub(crate) fn validated_limit(limit: u64) -> Result<u64, StorageError> {
    let limit = u32::try_from(limit).map_err(|_| {
        StorageError::InvalidInput("limit exceeds the maximum page size".to_owned())
    })?;
    Ok(PageLimit::new(limit)?.get() as u64)
}
