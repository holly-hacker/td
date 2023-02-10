//! The library that powers `td`, the graph-based todo manager.

#![warn(
    clippy::semicolon_if_nothing_returned,
    clippy::use_self,
    clippy::cloned_instead_of_copied
)]
#![warn(missing_docs, clippy::doc_markdown, clippy::must_use_candidate)]

pub mod database;
pub mod errors;

pub use time;
