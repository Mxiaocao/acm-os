use crate::LocalDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipationFacts {
    pub kind: String,
    pub local_date: String,
    pub local_time: String,
    pub accepted_count: Option<u32>,
    pub total_count: Option<u32>,
}

impl ParticipationFacts {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !matches!(self.kind.as_str(), "live" | "virtual") {
            return Err("invalid_participation_kind");
        }
        LocalDate::parse_iso(&self.local_date).map_err(|_| "invalid_participation_date")?;
        let time = self.local_time.as_bytes();
        if time.len() != 5
            || time[2] != b':'
            || ![time[0], time[1], time[3], time[4]]
                .iter()
                .all(u8::is_ascii_digit)
            || &self.local_time[..2] > "23"
            || &self.local_time[3..] > "59"
        {
            return Err("invalid_participation_time");
        }
        if self.total_count == Some(0)
            || self
                .accepted_count
                .zip(self.total_count)
                .is_some_and(|(ac, total)| ac > total)
        {
            return Err("invalid_participation_score");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn participation_validates_kind_local_start_and_optional_score() {
        let mut facts = ParticipationFacts {
            kind: "virtual".into(),
            local_date: "2026-10-15".into(),
            local_time: "23:30".into(),
            accepted_count: None,
            total_count: None,
        };
        assert!(facts.validate().is_ok());
        for time in ["24:00", "12:60", "1:30", "１２:３０"] {
            facts.local_time = time.into();
            assert!(facts.validate().is_err());
        }
        facts.local_time = "00:00".into();
        facts.local_date = "2026-02-30".into();
        assert!(facts.validate().is_err());
        facts.local_date = "2026-10-15".into();
        facts.kind = "practice".into();
        assert!(facts.validate().is_err());
        facts.kind = "live".into();
        facts.accepted_count = Some(3);
        facts.total_count = Some(2);
        assert!(facts.validate().is_err());
        facts.accepted_count = Some(0);
        assert!(facts.validate().is_ok());
        facts.total_count = Some(0);
        assert!(facts.validate().is_err());
    }
}
