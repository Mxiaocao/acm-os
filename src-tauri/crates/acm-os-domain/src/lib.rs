#![forbid(unsafe_code)]

pub const BOUNDARY_NAME: &str = "acm-os-domain";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeforcesContestIdentity {
    contest_id: u64,
}

impl CodeforcesContestIdentity {
    pub fn new(contest_id: u64) -> Result<Self, IdentityError> {
        if contest_id == 0 {
            return Err(IdentityError::InvalidContestId);
        }
        Ok(Self { contest_id })
    }

    pub const fn contest_id(&self) -> u64 {
        self.contest_id
    }

    pub const fn platform(&self) -> &'static str {
        "codeforces"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeforcesProblemIdentity {
    contest: CodeforcesContestIdentity,
    index: String,
}

impl CodeforcesProblemIdentity {
    pub fn new(contest: CodeforcesContestIdentity, index: impl Into<String>) -> Result<Self, IdentityError> {
        let index = index.into();
        let valid = !index.is_empty()
            && index.len() <= 8
            && index.bytes().all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
        if !valid {
            return Err(IdentityError::InvalidProblemIndex);
        }
        Ok(Self { contest, index })
    }

    pub fn contest(&self) -> &CodeforcesContestIdentity {
        &self.contest
    }

    pub fn index(&self) -> &str {
        &self.index
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {
    InvalidContestId,
    InvalidProblemIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codeforces_strong_identity_requires_a_positive_contest_id_and_canonical_index() {
        assert_eq!(
            CodeforcesContestIdentity::new(0),
            Err(IdentityError::InvalidContestId)
        );
        let contest = CodeforcesContestIdentity::new(1979).expect("contest identity");
        assert_eq!(contest.platform(), "codeforces");
        assert_eq!(contest.contest_id(), 1979);
        assert_eq!(
            CodeforcesProblemIdentity::new(contest.clone(), "a"),
            Err(IdentityError::InvalidProblemIndex)
        );
        assert_eq!(
            CodeforcesProblemIdentity::new(contest.clone(), "A/1"),
            Err(IdentityError::InvalidProblemIndex)
        );
        let problem = CodeforcesProblemIdentity::new(contest, "A1").expect("problem identity");
        assert_eq!(problem.index(), "A1");
        assert_eq!(problem.contest().contest_id(), 1979);
    }
}
