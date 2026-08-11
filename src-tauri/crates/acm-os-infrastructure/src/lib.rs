#![forbid(unsafe_code)]

pub mod codeforces;
mod file_binding;
mod markdown;
mod persistence;
mod safe_patch;

pub use persistence::{start_database, DatabaseRuntime};
