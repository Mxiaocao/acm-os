#![forbid(unsafe_code)]

pub mod codeforces;
mod file_binding;
mod markdown;
mod persistence;

pub use persistence::{start_database, DatabaseRuntime};
