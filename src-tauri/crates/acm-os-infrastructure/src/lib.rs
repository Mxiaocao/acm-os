#![forbid(unsafe_code)]

pub mod codeforces;
mod file_binding;
mod markdown;
mod persistence;
mod safe_patch;

pub use persistence::{start_database, DatabaseRuntime};

pub fn current_local_date() -> Result<acm_os_domain::LocalDate, ()> {
    let value = chrono::Local::now()
        .date_naive()
        .format("%Y-%m-%d")
        .to_string();
    acm_os_domain::LocalDate::parse_iso(&value).map_err(|_| ())
}
