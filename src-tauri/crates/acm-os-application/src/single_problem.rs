use crate::{ContestProblemSlotDraft, StatementAssetDraft, StatementSnapshotDraft};

#[derive(Debug, Clone)]
pub struct SingleProblemDraft {
    pub completion_date: Option<acm_os_domain::LocalDate>,
    pub platform: String,
    pub scope: String,
    pub key: String,
    pub title: String,
    pub source_url: String,
    pub rating: Option<u32>,
    pub source_html: String,
    pub sanitized_html: String,
    pub assets: Vec<StatementAssetDraft>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndependentProblemItem {
    pub imported_at_utc: String,
    pub statement_html: Option<String>,
    pub problem_id: i64,
    pub title: String,
    pub source_url: String,
    pub rating: Option<u32>,
    pub has_statement_snapshot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SingleProblemError { InvalidInput, Unavailable }
impl SingleProblemError {
    pub fn code(self) -> &'static str {
        match self { Self::InvalidInput => "single_problem_invalid_input", Self::Unavailable => "single_problem_unavailable" }
    }
}

impl SingleProblemDraft {
    pub fn manual(title: &str, source_url: &str, request_id: &str, rating: Option<u32>, statement: &str) -> Result<Self, SingleProblemError> {
        if title.trim().is_empty() || title.len() > 1000 || statement.trim().is_empty() || statement.len() > 2 * 1024 * 1024
            || rating == Some(0) || request_id.len() != 36
            || !request_id.chars().enumerate().all(|(i, c)| if [8,13,18,23].contains(&i) { c == '-' } else { c.is_ascii_hexdigit() }) {
            return Err(SingleProblemError::InvalidInput);
        }
        let source_url = if source_url.is_empty() { format!("acmos://local-problem/{request_id}") } else { source_url.to_owned() };
        let key = source_url.clone();
        let escaped = statement.trim().replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;").replace("\r\n", "\n").replace('\n', "<br>");
        Ok(Self {
            completion_date: None,
            platform: "independent".into(), scope: "source".into(), key,
            title: title.trim().into(), source_url, rating, source_html: statement.trim().into(),
            sanitized_html: format!("<div class=\"problem-statement manual-statement\"><p>{escaped}</p></div>"), assets: Vec::new(),
        })
    }

    pub fn from_codeforces(slot: &ContestProblemSlotDraft, snapshot: StatementSnapshotDraft) -> Result<Self, SingleProblemError> {
        if slot.problem != snapshot.problem { return Err(SingleProblemError::InvalidInput); }
        Ok(Self {
            completion_date: None,
            platform: "codeforces".into(), scope: slot.problem.contest().contest_id().to_string(), key: slot.problem.index().into(),
            title: slot.title.clone(), source_url: slot.source_url.clone(), rating: slot.rating,
            source_html: snapshot.source_html, sanitized_html: snapshot.sanitized_html, assets: snapshot.assets,
        })
    }
}

#[allow(async_fn_in_trait)]
pub trait SingleProblemPort {
    async fn import_single_problem(&self, draft: &SingleProblemDraft) -> Result<i64, SingleProblemError>;
    async fn independent_problems(&self) -> Result<Vec<IndependentProblemItem>, SingleProblemError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn single_problem_manual_accepts_no_link_and_escapes_markup() {
        let token = "018f0d8e-4a5b-7c6d-8e9f-0123456789ab";
        let draft = SingleProblemDraft::manual("Book problem", "", token, None, "<script>bad()</script>\n$ x < y $").unwrap();
        assert_eq!(draft.source_url, format!("acmos://local-problem/{token}"));
        assert!(!draft.sanitized_html.contains("<script>"));
        assert!(draft.sanitized_html.contains("&lt;script&gt;"));
        assert!(draft.sanitized_html.contains("<br>"));
        assert!(SingleProblemDraft::manual("", "", token, None, "text").is_err());
        assert!(SingleProblemDraft::manual("title", "", token, Some(0), "text").is_err());
        assert!(SingleProblemDraft::manual("title", "", "bad-token", None, "text").is_err());
    }
}
