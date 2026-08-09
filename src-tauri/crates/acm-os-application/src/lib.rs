#![forbid(unsafe_code)]

pub const BOUNDARY_NAME: &str = "acm-os-application";

pub struct FoundationStatus {
    pub status: &'static str,
    pub core: &'static str,
}

pub fn foundation_status() -> FoundationStatus {
    FoundationStatus {
        status: "ready",
        core: acm_os_domain::BOUNDARY_NAME,
    }
}
