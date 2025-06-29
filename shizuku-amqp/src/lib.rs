#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../../README.md")]

/// internal errors
pub mod error;

/// tools for rabbitmq
pub mod rabbitmq;

