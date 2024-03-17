#[cfg(feature = "validate")]
pub mod garde;
pub mod newtype;

use crate::workflow::definition::auth::Scheme;
use crate::workflow::definition::common::{ExecutionMode, InvocationMode};
use crate::workflow::definition::events::EventKind;
use crate::workflow::definition::functions::FunctionType;
use crate::workflow::definition::{CompletionType, OnComplete};

pub fn true_value() -> bool {
    true
}

pub fn false_value() -> bool {
    false
}

pub fn jq() -> String {
    "jq".to_string()
}

pub fn sequential() -> ExecutionMode {
    ExecutionMode::Sequential
}

pub fn parallel() -> ExecutionMode {
    ExecutionMode::Parallel
}

pub fn sync() -> InvocationMode {
    InvocationMode::Sync
}

pub fn terminate() -> OnComplete {
    OnComplete::Terminate
}

pub fn all_of() -> CompletionType {
    CompletionType::AllOf
}

pub fn basic() -> Scheme {
    Scheme::Basic
}

pub fn consumed() -> EventKind {
    EventKind::Consumed
}

pub fn rest() -> FunctionType {
    FunctionType::Rest
}
