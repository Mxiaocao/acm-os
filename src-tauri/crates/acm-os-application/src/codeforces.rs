use acm_os_domain::CodeforcesContestIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeforcesLocatorError {
    UnsupportedUrl,
}

/// Parses a Codeforces public contest locator without turning the adapter into
/// an arbitrary URL downloader. Remote request URLs are built later from this
/// strong identity, never copied from user input.
pub fn locate_public_contest(url: &str) -> Result<CodeforcesContestIdentity, CodeforcesLocatorError> {
    let trimmed = url.trim();
    let rest = trimmed
        .strip_prefix("https://codeforces.com/contest/")
        .or_else(|| trimmed.strip_prefix("https://www.codeforces.com/contest/"))
        .ok_or(CodeforcesLocatorError::UnsupportedUrl)?;
    let contest_id_text = rest.strip_suffix('/').unwrap_or(rest);
    let contest_id = (!contest_id_text.is_empty()
        && !contest_id_text.contains('/')
        && contest_id_text.bytes().all(|byte| byte.is_ascii_digit()))
        .then_some(contest_id_text)
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or(CodeforcesLocatorError::UnsupportedUrl)?;
    CodeforcesContestIdentity::new(contest_id).map_err(|_| CodeforcesLocatorError::UnsupportedUrl)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_accepts_only_supported_canonical_contest_urls() {
        assert_eq!(
            locate_public_contest(" https://codeforces.com/contest/1979/ "),
            Ok(CodeforcesContestIdentity::new(1979).expect("valid id"))
        );
        assert_eq!(
            locate_public_contest("https://codeforces.com/contest/2256"),
            Ok(CodeforcesContestIdentity::new(2256).expect("valid id"))
        );
        for value in [
            "http://codeforces.com/contest/1979",
            "https://codeforces.com/problemset/problem/1979/A",
            "https://evil.example/contest/1979",
            "https://codeforces.com/contest/1979/problem/A",
            "https://codeforces.com/contest/0",
        ] {
            assert_eq!(locate_public_contest(value), Err(CodeforcesLocatorError::UnsupportedUrl));
        }
    }
}
