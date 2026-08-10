#![forbid(unsafe_code)]

pub mod codeforces;
mod persistence;

pub use persistence::{start_database, DatabaseRuntime};
