//! Definition of a type that cannot be created.

/// A type that cannot be created.
///
/// Because the enum has no variant, a value of this type cannot exist. It is used to denote
/// something that can never happen - for example, error types that cannot exist if a certain
/// feature is disabled.
///
/// ## Comparison to `!`
///
/// Rust has a type called `never`, represented as [`!`]. Ideally, this is the type we would
/// use to represent a value that cannot exist; however, as of this writing, the `never` type
/// is still unstable. When the type is stabilized, it could be used instead.
#[derive(Debug)]
pub enum Impossible {}
