use super::*;

mod inscription;
mod outpoint;
mod transaction;
mod brc0;

pub(super) use {inscription::*, outpoint::*, transaction::*,brc0::*};

#[derive(Debug, thiserror::Error)]
pub enum OrdError {
  #[error("operation not found")]
  OperationNotFound,
  #[error("block not found")]
  BlockNotFound,
}
