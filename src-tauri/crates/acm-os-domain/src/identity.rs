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
    fn platform_key_enforces_provider_independent_structure() {
        let key = PlatformKey::new("CodeForces").expect("mixed case remains valid");
        assert_eq!(key.as_str(), "CodeForces");
        for invalid in ["", "   ", " codeforces", "codeforces ", "code\nforces"] {
            assert!(PlatformKey::new(invalid).is_err(), "accepted {invalid:?}");
        }
    }

    #[test]
    fn external_contest_key_is_opaque_and_exact() {
        assert_eq!(ExternalContestKey::new("1979").unwrap().as_str(), "1979");
        assert_eq!(
            ExternalContestKey::new("abc-2026-final").unwrap().as_str(),
            "abc-2026-final"
        );
        for invalid in ["", "\t", " 1979", "1979 ", "abc\u{7f}def"] {
            assert!(
                ExternalContestKey::new(invalid).is_err(),
                "accepted {invalid:?}"
            );
        }
    }

    #[test]
    fn contest_identity_includes_platform_and_external_key() {
        let identity = contest("codeforces", "1979");
        assert_eq!(identity.platform().as_str(), "codeforces");
        assert_eq!(identity.external_contest_key().as_str(), "1979");
        assert_eq!(identity, contest("codeforces", "1979"));
        assert_ne!(identity, contest("nowcoder", "1979"));
        let identities = HashSet::from([
            identity,
            contest("codeforces", "1979"),
            contest("nowcoder", "1979"),
        ]);
        assert_eq!(identities.len(), 2);
    }

    #[test]
    fn problem_identity_preserves_contest_namespace() {
        let first = ProblemIdentity::new(contest("alpha", "round-1"), "problem/a").unwrap();
        let same = ProblemIdentity::new(contest("alpha", "round-1"), "problem/a").unwrap();
        let other_contest = ProblemIdentity::new(contest("alpha", "round-2"), "problem/a").unwrap();
        assert_eq!(first, same);
        assert_ne!(first, other_contest);
        assert_eq!(first.external_problem_key(), "problem/a");
        assert_eq!(first.contest().platform().as_str(), "alpha");
        for invalid in ["", "   ", " A", "A ", "A\nB"] {
            assert!(ProblemIdentity::new(contest("alpha", "round-1"), invalid).is_err());
        }
    }
}
