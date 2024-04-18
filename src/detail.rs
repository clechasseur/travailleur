#[cfg(feature = "validate")]
pub mod garde;
pub mod newtype;

use crate::workflow::definition::auth::Scheme;
use crate::workflow::definition::common::{ExecutionMode, InvocationMode};
use crate::workflow::definition::events::EventKind;
use crate::workflow::definition::functions::FunctionType;
use crate::workflow::definition::{CompletionType, OnComplete};

pub trait OptFrom<T>: Sized {
    fn opt_from(value: T) -> Option<Self>;
}

pub trait IntoOpt<U>: Sized {
    fn into_opt(self) -> Option<U>;
}

impl<T, U> IntoOpt<U> for T
where
    U: OptFrom<T>,
{
    fn into_opt(self) -> Option<U> {
        U::opt_from(self)
    }
}

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

// A trait that is essentially a stub for `garde::Validate` (with `Context = ()`).
// If the `validate` feature is disabled, it's an empty trait.
// It's implemented for all types (that also implement `garde::Validate`, if needed).
#[cfg(feature = "validate")]
pub trait GardeValidate: ::garde::Validate<Context = ()> {}

#[cfg(feature = "validate")]
impl<T> GardeValidate for T where T: ::garde::Validate<Context = ()> {}

#[cfg(not(feature = "validate"))]
pub trait GardeValidate {}

#[cfg(not(feature = "validate"))]
impl<T> GardeValidate for T {}
