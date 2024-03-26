//! A Rust library implementing the [Serverless workflow] specification. In progress.
//!
//! Implements [v0.8] of the specification.
//!
//! [Serverless workflow]: https://serverlessworkflow.io/
//! [v0.8]: https://github.com/serverlessworkflow/specification/blob/v0.8/specification.md

// TODO re-enable once we're ready to document
// #![deny(missing_docs)]
// #![deny(rustdoc::missing_crate_level_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::private_intra_doc_links)]
#![cfg_attr(any(nightly_rustc, docsrs), feature(doc_cfg))]

pub mod cache;
pub(crate) mod detail;
pub mod error;
pub mod loader;
pub mod validation;
pub mod workflow;

pub use error::Error;
pub use error::Result;
