use super::*;

mod inscription;
mod outpoint;
mod transaction;
mod zeroindexer;

pub(super) use {inscription::*, outpoint::*, transaction::*, zeroindexer::*};

#[derive(Debug, thiserror::Error)]
pub enum OrdError {
  #[error("operation not found")]
  OperationNotFound,
  #[error("block not found")]
  BlockNotFound,
}
