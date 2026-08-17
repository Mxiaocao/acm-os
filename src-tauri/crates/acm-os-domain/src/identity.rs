#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenericIdentityError {
    Empty,
    SurroundingWhitespace,
    ControlCharacter,
}

fn validate_component(value: &str) -> Result<(), GenericIdentityError> {
    if value.is_empty() {
        return Err(GenericIdentityError::Empty);
    }
    if value.chars().any(char::is_control) {
        return Err(GenericIdentityError::ControlCharacter);
    }
    if value.trim() != value {
        return Err(GenericIdentityError::SurroundingWhitespace);
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PlatformKey(String);

impl PlatformKey {
    pub fn new(value: impl Into<String>) -> Result<Self, GenericIdentityError> {
        let value = value.into();
        validate_component(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternalContestKey(String);

impl ExternalContestKey {
    pub fn new(value: impl Into<String>) -> Result<Self, GenericIdentityError> {
        let value = value.into();
        validate_component(&value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContestIdentity {
    platform: PlatformKey,
    external_contest_key: ExternalContestKey,
}

impl ContestIdentity {
    pub fn new(platform: PlatformKey, external_contest_key: ExternalContestKey) -> Self {
        Self {
            platform,
            external_contest_key,
        }
    }

    pub fn platform(&self) -> &PlatformKey {
        &self.platform
    }

    pub fn external_contest_key(&self) -> &ExternalContestKey {
        &self.external_contest_key
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProblemIdentity {
    contest: ContestIdentity,
    external_problem_key: String,
}

impl ProblemIdentity {
    pub fn new(
        contest: ContestIdentity,
        external_problem_key: impl Into<String>,
    ) -> Result<Self, GenericIdentityError> {
        let external_problem_key = external_problem_key.into();
        validate_component(&external_problem_key)?;
        Ok(Self {
            contest,
            external_problem_key,
        })
    }

    pub fn contest(&self) -> &ContestIdentity {
        &self.contest
    }

    pub fn external_problem_key(&self) -> &str {
        &self.external_problem_key
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn platform(value: &str) -> PlatformKey {
        PlatformKey::new(value).expect("valid platform key")
    }

    fn contest(platform_key: &str, external_key: &str) -> ContestIdentity {
        ContestIdentity::new(
            platform(platform_key),
            ExternalContestKey::new(external_key).expect("valid external contest key"),
        )
    }

    #[test]
    fn platform_key_enforces_only_provider_independent_structure() {
        let key = PlatformKey::new("CodeForces").expect("mixed case remains valid");
        assert_eq!(key.as_str(), "CodeForces");

        for invalid in ["", "   ", " codeforces", "codeforces ", "code\nforces"] {
            assert!(PlatformKey::new(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn external_contest_key_is_an_exact_opaque_string() {
        let numeric = ExternalContestKey::new("1979").expect("numeric key");
        assert_eq!(numeric.as_str(), "1979");

        let opaque = ExternalContestKey::new("abc-2026-final").expect("opaque key");
        assert_eq!(opaque.as_str(), "abc-2026-final");

        let case_sensitive = ExternalContestKey::new("Final-A").expect("case-preserved key");
        assert_eq!(case_sensitive.as_str(), "Final-A");

        for invalid in ["", "\t", " 1979", "1979 ", "abc\u{7f}def"] {
            assert!(
                ExternalContestKey::new(invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
    }

    #[test]
    fn contest_strong_identity_contains_platform_and_external_key() {
        let identity = contest("codeforces", "1979");
        assert_eq!(identity.platform().as_str(), "codeforces");
        assert_eq!(identity.external_contest_key().as_str(), "1979");
        assert_eq!(identity, contest("codeforces", "1979"));
        assert_ne!(identity, contest("codeforces", "1980"));
        assert_ne!(identity, contest("nowcoder", "1979"));

        let identities = HashSet::from([
            identity.clone(),
            contest("codeforces", "1979"),
            contest("codeforces", "1980"),
            contest("nowcoder", "1979"),
        ]);
        assert_eq!(identities.len(), 3);
        assert!(identities.contains(&identity));
    }

    #[test]
    fn problem_identity_preserves_the_full_contest_namespace() {
        let first = ProblemIdentity::new(contest("alpha", "round-1"), "problem/a")
            .expect("provider-neutral problem key");
        let same =
            ProblemIdentity::new(contest("alpha", "round-1"), "problem/a").expect("same identity");
        let other_contest = ProblemIdentity::new(contest("alpha", "round-2"), "problem/a")
            .expect("same problem key under another contest");
        let other_platform = ProblemIdentity::new(contest("beta", "round-1"), "problem/a")
            .expect("same problem key under another platform");

        assert_eq!(first, same);
        assert_ne!(first, other_contest);
        assert_ne!(first, other_platform);
        assert_eq!(first.external_problem_key(), "problem/a");
        assert_eq!(first.contest().platform().as_str(), "alpha");

        let identities = HashSet::from([first.clone(), same, other_contest, other_platform]);
        assert_eq!(identities.len(), 3);

        for invalid in ["", "   ", " A", "A ", "A\nB"] {
            assert!(
                ProblemIdentity::new(contest("alpha", "round-1"), invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
    }
}
