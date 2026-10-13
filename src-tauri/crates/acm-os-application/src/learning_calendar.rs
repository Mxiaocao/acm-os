use acm_os_domain::LocalDate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningCalendarEntry {
    pub date: String,
    pub problem_id: i64,
    pub problem_title: String,
    pub kind: String,
    pub status: String,
    pub due_date: Option<String>,
    pub judgement: Option<String>,
}

#[allow(async_fn_in_trait)]
pub trait LearningCalendarPort {
    /// Read recorded learning and the next authoritative review due date only.
    /// Opening a calendar must never generate daily plans or advance scheduling.
    async fn load_learning_calendar(
        &self,
        start: LocalDate,
        end: LocalDate,
        today: LocalDate,
    ) -> Result<Vec<LearningCalendarEntry>, crate::TodaySnapshotError>;
}

pub fn calendar_month_range(year: u32, month: u32) -> Result<(LocalDate, LocalDate), ()> {
    if !(1..=9998).contains(&year) || !(1..=12).contains(&month) {
        return Err(());
    }
    let start = LocalDate::parse_iso(&format!("{year:04}-{month:02}-01")).map_err(|_| ())?;
    let (next_year, next_month) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    let end = LocalDate::parse_iso(&format!("{next_year:04}-{next_month:02}-01")).map_err(|_| ())?;
    Ok((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_month_range_handles_leap_months_and_year_boundaries() {
        let (start, end) = calendar_month_range(2024, 2).unwrap();
        assert_eq!(start.to_iso_string(), "2024-02-01");
        assert_eq!(end.to_iso_string(), "2024-03-01");
        let (_, end) = calendar_month_range(2026, 12).unwrap();
        assert_eq!(end.to_iso_string(), "2027-01-01");
        for (year, month) in [(0, 1), (10000, 1), (2026, 0), (2026, 13)] {
            assert!(calendar_month_range(year, month).is_err());
        }
    }
}
