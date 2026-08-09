#![forbid(unsafe_code)]

mod persistence;

pub use persistence::{start_database, DatabaseRuntime};
