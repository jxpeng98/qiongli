use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::ProjectError;

#[derive(Clone, Debug, Default)]
pub struct PortfolioCancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl PortfolioCancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    pub(crate) fn check(&self) -> Result<(), ProjectError> {
        if self.is_cancelled() {
            Err(ProjectError::OperationCancelled)
        } else {
            Ok(())
        }
    }
}
