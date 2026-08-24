use crate::storage::StorageError;

pub const MAX_PAGE_SIZE: u32 = 500;
pub const DEFAULT_PAGE_SIZE: u32 = 100;
pub const MAX_TRAVERSAL_DEPTH: u32 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageLimit(u32);

impl PageLimit {
    pub fn new(limit: u32) -> Result<Self, StorageError> {
        if !(1..=MAX_PAGE_SIZE).contains(&limit) {
            return Err(StorageError::InvalidInput(format!(
                "limit must be between 1 and {MAX_PAGE_SIZE}"
            )));
        }
        Ok(Self(limit))
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl Default for PageLimit {
    fn default() -> Self {
        Self(DEFAULT_PAGE_SIZE)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TraversalDepth(u32);

impl TraversalDepth {
    pub fn new(depth: u32) -> Result<Self, StorageError> {
        if !(1..=MAX_TRAVERSAL_DEPTH).contains(&depth) {
            return Err(StorageError::InvalidInput(format!(
                "max_depth must be between 1 and {MAX_TRAVERSAL_DEPTH}"
            )));
        }
        Ok(Self(depth))
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}
