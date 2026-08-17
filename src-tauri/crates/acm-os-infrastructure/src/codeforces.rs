use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

use acm_os_application::{
    ContestImportContractError, ContestImportDraft, ContestImportSource, ContestImportSourceError,
    ContestProblemSlotDraft, StatementAssetDraft, StatementSnapshotDraft,
};
use acm_os_domain::{CodeforcesContestIdentity, CodeforcesProblemIdentity};
use chrono::{DateTime, SecondsFormat, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureAdapterError {
    MetadataInvalid,
    ContestIdentityMismatch,
    ManifestInvalid(ContestImportContractError),
    MissingStatement(String),
    MissingAsset(String),
    UnsafeAssetUrl(String),
    StatementInvalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatementFixture {
    pub html: String,
    /// Exact, normalized Codeforces source URL -> captured local bytes.
    pub assets: BTreeMap<String, FixtureAsset>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixtureAsset {
    pub media_type: String,
    pub bytes: Vec<u8>,
}

const STATEMENT_MAX_BYTES: usize = 2 * 1024 * 1024;
const ASSET_MAX_BYTES: usize = 4 * 1024 * 1024;
const CONTEST_METADATA_MAX_BYTES: usize = 512 * 1024;
const CODEFORCES_TIMEOUT: Duration = Duration::from_secs(20);

/// The real network adapter accepts strong identities only. It constructs each
/// request URL itself, so a pasted contest locator can never become an
/// arbitrary downloader URL. Its methods perform no SQLite work.
pub struct CodeforcesHttpAdapter {
    client: reqwest::Client,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodeforcesHttpError {
    ClientUnavailable,
    RequestFailed,
    UnexpectedStatus { status: u16, diagnostic: String },
    ResponseTooLarge,
    InvalidUtf8,
    UnsafeAssetUrl(String),
}

impl CodeforcesHttpAdapter {
    pub fn new() -> Result<Self, CodeforcesHttpError> {
        // reqwest's rustls-no-provider feature deliberately leaves provider
        // selection to us; ring avoids the AWS-LC/NASM toolchain path.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let client = reqwest::Client::builder()
            .timeout(CODEFORCES_TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("ACM-OS/0.1 contest-import")
            .build()
            .map_err(|_| CodeforcesHttpError::ClientUnavailable)?;
        Ok(Self { client })
    }

    pub async fn fetch_contest_metadata(
        &self,
        contest: &CodeforcesContestIdentity,
    ) -> Result<String, CodeforcesHttpError> {
        let mut response = self.send(&contest_api_url(contest)).await?;
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| CodeforcesHttpError::RequestFailed)?
        {
            let next_size = body
                .len()
                .checked_add(chunk.len())
                .ok_or(CodeforcesHttpError::ResponseTooLarge)?;
            if next_size > CONTEST_METADATA_MAX_BYTES {
                return Err(CodeforcesHttpError::ResponseTooLarge);
            }
            body.extend_from_slice(&chunk);
            if let Some(metadata) = standings_metadata_without_rows(&body) {
                return String::from_utf8(metadata).map_err(|_| CodeforcesHttpError::InvalidUtf8);
            }
        }
        String::from_utf8(body).map_err(|_| CodeforcesHttpError::InvalidUtf8)
    }

    pub async fn fetch_problem_statement(
        &self,
        problem: &CodeforcesProblemIdentity,
    ) -> Result<String, CodeforcesHttpError> {
        self.fetch_text(&problem_page_fetch_url(problem), STATEMENT_MAX_BYTES)
            .await
    }

    /// Asset addresses originate in untrusted statement HTML but are validated
    /// to Codeforces before a request is issued.
    pub async fn fetch_statement_asset(
        &self,
        source_url: &str,
    ) -> Result<Vec<u8>, CodeforcesHttpError> {
        let source_url = trusted_asset_url_http(source_url)?;
        self.fetch_bytes(&source_url, ASSET_MAX_BYTES).await
    }

    async fn fetch_text(&self, url: &str, maximum: usize) -> Result<String, CodeforcesHttpError> {
        let bytes = self.fetch_bytes(url, maximum).await?;
        String::from_utf8(bytes).map_err(|_| CodeforcesHttpError::InvalidUtf8)
    }

    async fn fetch_bytes(&self, url: &str, maximum: usize) -> Result<Vec<u8>, CodeforcesHttpError> {
        let response = self.send(url).await?;
        if response
            .content_length()
            .is_some_and(|size| size > maximum as u64)
        {
            return Err(CodeforcesHttpError::ResponseTooLarge);
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|_| CodeforcesHttpError::RequestFailed)?;
        if bytes.len() > maximum {
            return Err(CodeforcesHttpError::ResponseTooLarge);
        }
        Ok(bytes.to_vec())
    }

    async fn send(&self, url: &str) -> Result<reqwest::Response, CodeforcesHttpError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|_| CodeforcesHttpError::RequestFailed)?;
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let diagnostic = response
                .text()
                .await
                .map(|body| body.chars().take(512).collect())
                .unwrap_or_else(|_| "response body unavailable".to_owned());
            return Err(CodeforcesHttpError::UnexpectedStatus { status, diagnostic });
        }
        Ok(response)
    }
}

/// Codeforces places `contest` and `problems` before an unbounded `rows` array.
/// ACM-OS needs only the manifest fields, so stop at that top-level result key
/// instead of downloading every participant's standings row.
fn standings_metadata_without_rows(body: &[u8]) -> Option<Vec<u8>> {
    let mut object_depth = 0usize;
    let mut cursor = 0usize;
    while cursor < body.len() {
        match body[cursor] {
            b'{' => {
                object_depth += 1;
                cursor += 1;
            }
            b'}' => {
                object_depth = object_depth.checked_sub(1)?;
                cursor += 1;
            }
            b'"' => {
                let start = cursor + 1;
                cursor = start;
                let mut escaped = false;
                while cursor < body.len() {
                    let byte = body[cursor];
                    if escaped {
                        escaped = false;
                    } else if byte == b'\\' {
                        escaped = true;
                    } else if byte == b'"' {
                        break;
                    }
                    cursor += 1;
                }
                if cursor == body.len() {
                    return None;
                }
                if object_depth == 2 && &body[start..cursor] == b"rows" {
                    let mut colon = cursor + 1;
                    while colon < body.len() && body[colon].is_ascii_whitespace() {
                        colon += 1;
                    }
                    if colon == body.len() {
                        return None;
                    }
                    if body[colon] != b':' {
                        cursor += 1;
                        continue;
                    }
                    let mut metadata = body[..=colon].to_vec();
                    metadata.extend_from_slice(b"[]}}");
                    return Some(metadata);
                }
                cursor += 1;
            }
            _ => cursor += 1,
        }
    }
    None
}

impl ContestImportSource for CodeforcesHttpAdapter {
    async fn fetch_manifest(
        &self,
        contest: &CodeforcesContestIdentity,
    ) -> Result<ContestImportDraft, ContestImportSourceError> {
        let metadata = self
            .fetch_contest_metadata(contest)
            .await
            .map_err(map_http_error)?;
        manifest_from_api_json(contest.clone(), &metadata)
            .map_err(|_| ContestImportSourceError::InvalidRemoteData)
    }

    async fn fetch_snapshot(
        &self,
        problem: &CodeforcesProblemIdentity,
    ) -> Result<StatementSnapshotDraft, ContestImportSourceError> {
        let page = self
            .fetch_problem_statement(problem)
            .await
            .map_err(map_http_error)?;
        let statement =
            extract_problem_statement(&page).ok_or(ContestImportSourceError::InvalidRemoteData)?;
        let mut assets = BTreeMap::new();
        for source_url in statement_asset_urls(statement) {
            let bytes = self
                .fetch_statement_asset(&source_url)
                .await
                .map_err(map_http_error)?;
            assets.insert(
                source_url,
                FixtureAsset {
                    media_type: "application/octet-stream".to_owned(),
                    bytes,
                },
            );
        }
        let fixture = StatementFixture {
            html: statement.to_owned(),
            assets,
        };
        let (sanitized_html, assets) = sanitize_and_localize_statement(problem, &fixture)
            .map_err(|_| ContestImportSourceError::InvalidRemoteData)?;
        Ok(StatementSnapshotDraft {
            problem: problem.clone(),
            source_html: statement.to_owned(),
            sanitized_html,
            assets,
        })
    }
}

fn map_http_error(error: CodeforcesHttpError) -> ContestImportSourceError {
    match error {
        CodeforcesHttpError::InvalidUtf8 | CodeforcesHttpError::UnsafeAssetUrl(_) => {
            ContestImportSourceError::InvalidRemoteData
        }
        _ => ContestImportSourceError::Unavailable,
    }
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope {
    status: String,
    result: ApiStandingsResult,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiStandingsResult {
    contest: ApiContest,
    problems: Vec<ApiProblem>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiContest {
    id: u64,
    name: String,
    start_time_seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiProblem {
    contest_id: u64,
    index: String,
    name: String,
    rating: Option<u32>,
}

pub fn contest_api_url(contest: &CodeforcesContestIdentity) -> String {
    format!(
        "https://codeforces.com/api/contest.standings?contestId={}",
        contest.contest_id()
    )
}

pub fn problem_page_url(problem: &CodeforcesProblemIdentity) -> String {
    format!(
        "https://codeforces.com/contest/{}/problem/{}",
        problem.contest().contest_id(),
        problem.index()
    )
}

fn problem_page_fetch_url(problem: &CodeforcesProblemIdentity) -> String {
    format!("{}?locale=en", problem_page_url(problem))
}

/// Deterministic adapter boundary for fixed official API, HTML, and asset
/// fixtures. HTTP is intentionally not part of this function and therefore
/// can never run while a SQLite import transaction is open.
pub fn build_fixture_import(
    contest: CodeforcesContestIdentity,
    api_json: &str,
    statements: &BTreeMap<String, StatementFixture>,
) -> Result<(ContestImportDraft, Vec<StatementSnapshotDraft>), FixtureAdapterError> {
    let draft = manifest_from_api_json(contest, api_json)?;
    let mut snapshots = Vec::with_capacity(draft.slots.len());
    for slot in &draft.slots {
        let fixture = statements.get(slot.problem.index()).ok_or_else(|| {
            FixtureAdapterError::MissingStatement(slot.problem.index().to_owned())
        })?;
        let (sanitized_html, assets) = sanitize_and_localize_statement(&slot.problem, fixture)?;
        snapshots.push(StatementSnapshotDraft {
            problem: slot.problem.clone(),
            source_html: fixture.html.clone(),
            sanitized_html,
            assets,
        });
    }
    Ok((draft, snapshots))
}

fn manifest_from_api_json(
    contest: CodeforcesContestIdentity,
    api_json: &str,
) -> Result<ContestImportDraft, FixtureAdapterError> {
    let api: ApiEnvelope =
        serde_json::from_str(api_json).map_err(|_| FixtureAdapterError::MetadataInvalid)?;
    if api.status != "OK" || api.result.contest.id != contest.contest_id() {
        return Err(FixtureAdapterError::ContestIdentityMismatch);
    }
    if api.result.contest.name.trim().is_empty() {
        return Err(FixtureAdapterError::MetadataInvalid);
    }
    let mut seen = HashSet::new();
    let mut slots = Vec::with_capacity(api.result.problems.len());
    for (offset, problem_metadata) in api.result.problems.iter().enumerate() {
        if problem_metadata.contest_id != contest.contest_id()
            || problem_metadata.name.trim().is_empty()
        {
            return Err(FixtureAdapterError::MetadataInvalid);
        }
        let problem =
            CodeforcesProblemIdentity::new(contest.clone(), problem_metadata.index.clone())
                .map_err(|_| FixtureAdapterError::MetadataInvalid)?;
        if !seen.insert(problem.clone()) {
            return Err(FixtureAdapterError::MetadataInvalid);
        }
        slots.push(ContestProblemSlotDraft {
            ordinal: offset as u32 + 1,
            source_url: problem_page_url(&problem),
            problem,
            title: problem_metadata.name.clone(),
            rating: problem_metadata.rating,
        });
    }
    let contest_source_url = format!("https://codeforces.com/contest/{}", contest.contest_id());
    let starts_at_utc = api
        .result
        .contest
        .start_time_seconds
        .map(canonical_utc_timestamp)
        .transpose()?;
    let draft = ContestImportDraft::validated(
        contest,
        api.result.contest.name,
        contest_source_url,
        starts_at_utc,
        slots,
    )
    .map_err(FixtureAdapterError::ManifestInvalid)?;

    Ok(draft)
}

fn canonical_utc_timestamp(seconds: i64) -> Result<String, FixtureAdapterError> {
    DateTime::<Utc>::from_timestamp(seconds, 0)
        .map(|timestamp| timestamp.to_rfc3339_opts(SecondsFormat::Secs, true))
        .ok_or(FixtureAdapterError::MetadataInvalid)
}

fn extract_problem_statement(page: &str) -> Option<&str> {
    let start = page.find("<div class=\"problem-statement\"")?;
    let mut depth = 0usize;
    let mut cursor = start;
    loop {
        let next_open = page[cursor..].find("<div").map(|offset| cursor + offset);
        let next_close = page[cursor..].find("</div>").map(|offset| cursor + offset);
        match (next_open, next_close) {
            (Some(open), Some(close)) if open < close => {
                depth += 1;
                cursor = page[open..].find('>')? + open + 1;
            }
            (_, Some(close)) => {
                depth = depth.checked_sub(1)?;
                cursor = close + "</div>".len();
                if depth == 0 {
                    return Some(&page[start..cursor]);
                }
            }
            _ => return None,
        }
    }
}

fn statement_asset_urls(statement: &str) -> Vec<String> {
    let mut urls = Vec::new();
    let mut cursor = 0usize;
    while let Some((start, end, source)) = next_attribute_value(&statement[cursor..], "src") {
        let _ = (start, end);
        let source_url = match trusted_asset_url_http(&source) {
            Ok(url) => url,
            Err(_) => return Vec::new(),
        };
        if !urls.contains(&source_url) {
            urls.push(source_url);
        }
        cursor += end + 1;
    }
    urls
}

pub fn sanitize_and_localize_statement(
    problem: &CodeforcesProblemIdentity,
    fixture: &StatementFixture,
) -> Result<(String, Vec<StatementAssetDraft>), FixtureAdapterError> {
    if !fixture.html.contains("problem-statement") {
        return Err(FixtureAdapterError::StatementInvalid(
            "problem statement root missing".to_owned(),
        ));
    }
    let mut output = remove_dangerous_elements(&fixture.html);
    output = strip_event_attributes(&output);
    output = rewrite_unsafe_links(&output);

    let mut assets = Vec::new();
    let mut ordinal = 0usize;
    let mut cursor = 0usize;
    while let Some((relative_start, relative_end, source)) =
        next_attribute_value(&output[cursor..], "src")
    {
        let start = cursor + relative_start;
        let end = cursor + relative_end;
        let source_url = trusted_asset_url(&source)?;
        let asset = fixture
            .assets
            .get(&source_url)
            .ok_or_else(|| FixtureAdapterError::MissingAsset(source_url.clone()))?;
        ordinal += 1;
        let local_ref = format!(
            "acm-os-asset://codeforces/{}/{}/{}",
            problem.contest().contest_id(),
            problem.index(),
            ordinal
        );
        output.replace_range(start..end, &local_ref);
        cursor = start + local_ref.len() + 1;
        assets.push(StatementAssetDraft {
            local_ref,
            media_type: asset.media_type.clone(),
            bytes: asset.bytes.clone(),
        });
    }
    Ok((output, assets))
}

fn trusted_asset_url(source: &str) -> Result<String, FixtureAdapterError> {
    if source.starts_with('/') {
        return Ok(format!("https://codeforces.com{source}"));
    }
    if source.starts_with("https://codeforces.com/")
        || source.starts_with("https://www.codeforces.com/")
        || source.starts_with("https://espresso.codeforces.com/")
    {
        return Ok(source.to_owned());
    }
    Err(FixtureAdapterError::UnsafeAssetUrl(source.to_owned()))
}

fn trusted_asset_url_http(source: &str) -> Result<String, CodeforcesHttpError> {
    if source.starts_with('/') {
        return Ok(format!("https://codeforces.com{source}"));
    }
    if source.starts_with("https://codeforces.com/")
        || source.starts_with("https://www.codeforces.com/")
        || source.starts_with("https://espresso.codeforces.com/")
    {
        return Ok(source.to_owned());
    }
    Err(CodeforcesHttpError::UnsafeAssetUrl(source.to_owned()))
}

fn remove_dangerous_elements(input: &str) -> String {
    let mut output = input.to_owned();
    for tag in ["script", "style", "iframe", "object", "embed", "link"] {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        while let Some(start) = output.to_ascii_lowercase().find(&open) {
            let tail = &output[start..];
            let Some(close_offset) = tail.to_ascii_lowercase().find(&close) else {
                let Some(end_offset) = tail.find('>') else {
                    break;
                };
                output.replace_range(start..start + end_offset + 1, "");
                continue;
            };
            output.replace_range(start..start + close_offset + close.len(), "");
        }
    }
    output
}

fn strip_event_attributes(input: &str) -> String {
    let mut output = input.to_owned();
    for event in ["onclick", "onload", "onerror", "onmouseover", "onfocus"] {
        while let Some((start, end, _)) = next_attribute_value(&output, event) {
            let attribute_start = output[..start]
                .rfind(|character: char| character.is_ascii_whitespace())
                .unwrap_or(start);
            output.replace_range(attribute_start..end + 1, "");
        }
    }
    output
}

fn rewrite_unsafe_links(input: &str) -> String {
    let mut output = input.to_owned();
    loop {
        let Some((start, end, value)) = next_attribute_value(&output, "href") else {
            break;
        };
        if value
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("javascript:")
            || value.trim_start().to_ascii_lowercase().starts_with("data:")
        {
            output.replace_range(start..end, "#");
        } else {
            let next_start = end + 1;
            let Some(relative) = output[next_start..].to_ascii_lowercase().find("href=") else {
                break;
            };
            let prefix = output[..next_start + relative].to_owned();
            let suffix = rewrite_unsafe_links(&output[next_start + relative..]);
            return format!("{prefix}{suffix}");
        }
    }
    output
}

fn next_attribute_value(input: &str, target: &str) -> Option<(usize, usize, String)> {
    let lower = input.to_ascii_lowercase();
    let needle = format!("{target}=");
    let start = lower.find(&needle)? + needle.len();
    let quote = input.as_bytes().get(start).copied()?;
    if quote != b'\"' && quote != b'\'' {
        return None;
    }
    let value_start = start + 1;
    let end = value_start + input[value_start..].find(quote as char)?;
    Some((value_start, end, input[value_start..end].to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contest() -> CodeforcesContestIdentity {
        CodeforcesContestIdentity::new(1979).expect("contest")
    }
    fn html(image: &str) -> String {
        format!("<div class=\"problem-statement\"><script>bad()</script><p onclick=\"bad()\">Text</p><a href=\"javascript:bad()\">bad</a><img src=\"{image}\"></div>")
    }
    fn metadata() -> &'static str {
        r#"{"status":"OK","result":{"contest":{"id":1979,"name":"Round","startTimeSeconds":1710000000},"problems":[{"contestId":1979,"index":"A","name":"Alpha","rating":800},{"contestId":1979,"index":"B","name":"Beta","rating":900}]}}"#
    }
    fn fixtures() -> BTreeMap<String, StatementFixture> {
        let asset = FixtureAsset {
            media_type: "image/png".to_owned(),
            bytes: vec![1, 2, 3],
        };
        let mut result = BTreeMap::new();
        for index in ["A", "B"] {
            result.insert(
                index.to_owned(),
                StatementFixture {
                    html: html("/predownloaded/image.png"),
                    assets: BTreeMap::from([(
                        "https://codeforces.com/predownloaded/image.png".to_owned(),
                        asset.clone(),
                    )]),
                },
            );
        }
        result
    }

    #[test]
    fn fixture_adapter_builds_a_complete_ordered_manifest_and_local_snapshots() {
        let (draft, snapshots) =
            build_fixture_import(contest(), metadata(), &fixtures()).expect("fixture import");
        assert_eq!(draft.starts_at_utc.as_deref(), Some("2024-03-09T16:00:00Z"));
        assert_eq!(draft.slots.len(), 2);
        assert_eq!(draft.slots[0].problem.index(), "A");
        assert_eq!(
            contest_api_url(&draft.contest),
            "https://codeforces.com/api/contest.standings?contestId=1979"
        );
        assert_eq!(snapshots.len(), 2);
        assert!(!snapshots[0].sanitized_html.contains("<script"));
        assert!(!snapshots[0].sanitized_html.contains("onclick"));
        assert!(snapshots[0].sanitized_html.contains("href=\"#\""));
        assert!(snapshots[0]
            .sanitized_html
            .contains("acm-os-asset://codeforces/1979/A/1"));
        assert_eq!(snapshots[0].assets.len(), 1);
    }

    #[test]
    fn fixture_adapter_keeps_a_missing_start_time_unassigned() {
        let metadata = metadata().replace(",\"startTimeSeconds\":1710000000", "");
        let (draft, _) =
            build_fixture_import(contest(), &metadata, &fixtures()).expect("fixture import");

        assert_eq!(draft.starts_at_utc, None);
    }

    #[test]
    fn fixture_adapter_rejects_identity_and_missing_statement_or_asset_failures() {
        let mismatched = metadata().replace("\"id\":1979", "\"id\":1980");
        assert_eq!(
            build_fixture_import(contest(), &mismatched, &fixtures()),
            Err(FixtureAdapterError::ContestIdentityMismatch)
        );
        let mut missing = fixtures();
        missing.remove("B");
        assert_eq!(
            build_fixture_import(contest(), metadata(), &missing),
            Err(FixtureAdapterError::MissingStatement("B".to_owned()))
        );
        let fixture = StatementFixture {
            html: html("https://evil.example/x.png"),
            assets: BTreeMap::new(),
        };
        let problem = CodeforcesProblemIdentity::new(contest(), "A").expect("problem");
        assert_eq!(
            sanitize_and_localize_statement(&problem, &fixture),
            Err(FixtureAdapterError::UnsafeAssetUrl(
                "https://evil.example/x.png".to_owned()
            ))
        );
    }

    #[test]
    fn real_adapter_url_construction_is_identity_bound_and_assets_stay_codeforces_only() {
        let problem = CodeforcesProblemIdentity::new(contest(), "A").expect("problem");
        assert_eq!(
            problem_page_url(&problem),
            "https://codeforces.com/contest/1979/problem/A"
        );
        assert_eq!(
            problem_page_fetch_url(&problem),
            "https://codeforces.com/contest/1979/problem/A?locale=en"
        );
        assert_eq!(
            trusted_asset_url_http("/predownloaded/asset.png"),
            Ok("https://codeforces.com/predownloaded/asset.png".to_owned())
        );
        assert_eq!(
            trusted_asset_url_http("https://espresso.codeforces.com/asset.png"),
            Ok("https://espresso.codeforces.com/asset.png".to_owned())
        );
        assert_eq!(
            trusted_asset_url_http("https://espresso.codeforces.com.evil.example/asset.png"),
            Err(CodeforcesHttpError::UnsafeAssetUrl(
                "https://espresso.codeforces.com.evil.example/asset.png".to_owned()
            ))
        );
        assert_eq!(
            trusted_asset_url_http("https://evil.example/asset.png"),
            Err(CodeforcesHttpError::UnsafeAssetUrl(
                "https://evil.example/asset.png".to_owned()
            ))
        );
    }

    #[test]
    fn statement_asset_urls_normalize_and_deduplicate_codeforces_sources() {
        let html = "<div class=\"problem-statement\"><img src=\"/x.png\"><img src=\"/x.png\"><img src=\"https://espresso.codeforces.com/y.png\"></div>";
        assert_eq!(
            statement_asset_urls(html),
            vec![
                "https://codeforces.com/x.png",
                "https://espresso.codeforces.com/y.png"
            ]
        );
        assert!(statement_asset_urls("<img src=\"https://evil.example/x.png\">").is_empty());
    }

    #[test]
    fn statement_extraction_keeps_nested_problem_statement_content() {
        let page = "<main><div class=\"problem-statement\"><div><p>Input</p></div><div>Output</div></div></main>";
        assert_eq!(
            extract_problem_statement(page),
            Some("<div class=\"problem-statement\"><div><p>Input</p></div><div>Output</div></div>")
        );
    }

    #[test]
    fn standings_metadata_stops_before_unbounded_participant_rows() {
        let prefix = br#"{"status":"OK","result":{"contest":{"id":1979,"name":"Round","startTimeSeconds":1710000000},"problems":[{"contestId":1979,"index":"A","name":"Arrow rows are harmless","rating":800}],"ro"#;
        assert_eq!(standings_metadata_without_rows(prefix), None);

        let response = br#"{"status":"OK","result":{"contest":{"id":1979,"name":"Round","startTimeSeconds":1710000000},"problems":[{"contestId":1979,"index":"A","name":"Arrow rows are harmless","rating":800}],"rows":[{"party":{"contestId":1979},"points":1},{"party":{"contestId":1979},"points":2}]}}"#;
        let metadata = standings_metadata_without_rows(response).expect("manifest prefix");
        let metadata = String::from_utf8(metadata).expect("UTF-8 metadata");
        assert!(!metadata.contains("party"));
        let manifest = manifest_from_api_json(contest(), &metadata).expect("manifest");
        assert_eq!(manifest.slots.len(), 1);
        assert_eq!(manifest.slots[0].problem.index(), "A");
    }

    #[tokio::test]
    #[ignore = "release-only real Codeforces smoke; requires live network"]
    async fn real_codeforces_metadata_smoke() {
        let adapter = CodeforcesHttpAdapter::new().expect("HTTP adapter");
        // Contest 2256 reproduces the large unbounded standings response that
        // must be truncated after the manifest fields during release smoke.
        let contest = CodeforcesContestIdentity::new(2256).expect("contest identity");
        let metadata = adapter
            .fetch_contest_metadata(&contest)
            .await
            .expect("Codeforces metadata");
        assert!(metadata.contains("\"status\":\"OK\""));
        assert!(metadata.contains("\"contestId\":2256"));
        assert!(!metadata.contains("\"party\""));

        let problem = CodeforcesProblemIdentity::new(contest, "C").expect("problem identity");
        let snapshot = adapter
            .fetch_snapshot(&problem)
            .await
            .expect("Codeforces 2256C snapshot");
        assert!(snapshot
            .sanitized_html
            .contains("acm-os-asset://codeforces/2256/C/1"));
        assert!(!snapshot.assets.is_empty());
    }
}
