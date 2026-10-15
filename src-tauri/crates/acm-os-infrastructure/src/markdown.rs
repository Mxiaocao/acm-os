use std::collections::BTreeMap;

use acm_os_application::{
    KnownMarkdownSection, MarkdownDocumentSection, MarkdownParseWarning, ProblemMarkdownProjection, SolutionRoute,
};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};

const KNOWN_SECTIONS: [&str; 6] = ["前置知识", "题解", "额外题目", "Hints", "思路", "代码"];

#[derive(Debug)]
struct Heading {
    level: HeadingLevel,
    name: String,
    start: usize,
    heading_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReviewHelpContent {
    pub title: String,
    pub markdown: String,
}

pub(crate) fn parse_problem_markdown(
    markdown: &str,
    content_digest: String,
) -> ProblemMarkdownProjection {
    let headings = collect_headings(markdown);
    let mut known_sections = Vec::new();
    let mut solution_routes = Vec::new();
    let mut document_sections = Vec::new();
    let mut counts = BTreeMap::<String, usize>::new();

    for (index, heading) in headings.iter().enumerate() {
        let end_offset = section_end(&headings, index, markdown.len());
        if heading.level == HeadingLevel::H2 {
            let content_markdown = markdown[heading.heading_end..end_offset].trim().to_owned();
            // Methods retain their own modal bodies; only the solution preamble is inline.
            let body_end = if heading.name == "题解" {
                headings.iter().find(|child| child.level == HeadingLevel::H3 && child.start >= heading.heading_end && child.start < end_offset).map_or(end_offset, |child| child.start)
            } else { end_offset };
            document_sections.push(MarkdownDocumentSection {
                name: heading.name.clone(), start_offset: heading.start, end_offset,
                content_html: render_markdown(markdown[heading.heading_end..body_end].trim()), content_markdown,
            });
        }
        if heading.level == HeadingLevel::H2 && KNOWN_SECTIONS.contains(&heading.name.as_str()) {
            *counts.entry(heading.name.clone()).or_default() += 1;
            known_sections.push(KnownMarkdownSection {
                name: heading.name.clone(),
                start_offset: heading.start,
                end_offset,
            });
        }
        if heading.level == HeadingLevel::H3
            && direct_parent_h2(&headings, index).is_some_and(|name| name == "题解")
        {
            let content_markdown = markdown[heading.heading_end..end_offset].trim().to_owned();
            let content_html = render_markdown(&content_markdown);
            solution_routes.push(SolutionRoute {
                name: heading.name.clone(),
                start_offset: heading.start,
                end_offset,
                content_markdown,
                content_html,
            });
        }
    }

    let warnings = counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(name, count)| MarkdownParseWarning::DuplicateKnownSection { name, count })
        .collect();

    ProblemMarkdownProjection {
        content_digest,
        introduction_html: render_markdown(introduction(markdown, &headings)),
        document_sections,
        known_sections,
        solution_routes,
        warnings,
    }
}

fn render_markdown(markdown: &str) -> String {
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, Parser::new_ext(markdown,
        pulldown_cmark::Options::ENABLE_TABLES | pulldown_cmark::Options::ENABLE_STRIKETHROUGH | pulldown_cmark::Options::ENABLE_MATH));
    html
}

fn introduction<'a>(markdown: &'a str, headings: &[Heading]) -> &'a str {
    let end = headings.iter().find(|heading| heading.level == HeadingLevel::H2).map_or(markdown.len(), |heading| heading.start);
    let title_end = headings.iter().find(|heading| heading.level == HeadingLevel::H1 && heading.start < end).map(|heading| heading.heading_end);
    let metadata_end = if markdown.starts_with("---\n") || markdown.starts_with("---\r\n") {
        let first_line = markdown.find('\n').unwrap_or(0) + 1;
        let mut offset = first_line;
        let mut result = first_line;
        for line in markdown[first_line..].split_inclusive('\n') {
            offset += line.len();
            if line.trim() == "---" { result = offset; break; }
        }
        result
    } else { 0 };
    markdown[title_end.unwrap_or(metadata_end).min(end)..end].trim()
}

fn collect_headings(markdown: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut current: Option<Heading> = None;
    for (event, range) in Parser::new_ext(markdown, pulldown_cmark::Options::ENABLE_YAML_STYLE_METADATA_BLOCKS).into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current = Some(Heading {
                    level,
                    name: String::new(),
                    start: range.start,
                    heading_end: range.end,
                });
            }
            Event::Text(text) | Event::Code(text) => {
                if let Some(heading) = &mut current {
                    heading.name.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(heading) = &mut current {
                    heading.name.push(' ');
                }
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some(mut heading) = current.take() {
                    heading.name = heading.name.trim().to_owned();
                    heading.heading_end = range.end;
                    headings.push(heading);
                }
            }
            _ => {}
        }
    }
    headings
}

fn section_end(headings: &[Heading], index: usize, markdown_len: usize) -> usize {
    let level = headings[index].level as u8;
    headings[index + 1..]
        .iter()
        .find(|heading| heading.level as u8 <= level)
        .map_or(markdown_len, |heading| heading.start)
        .max(headings[index].heading_end)
}

fn direct_parent_h2(headings: &[Heading], index: usize) -> Option<&str> {
    headings[..index]
        .iter()
        .rev()
        .find(|heading| (heading.level as u8) < (HeadingLevel::H3 as u8))
        .filter(|heading| heading.level == HeadingLevel::H2)
        .map(|heading| heading.name.as_str())
}

pub(crate) fn review_help_content(
    markdown: &str,
    level: acm_os_domain::ReviewHelpLevel,
) -> Option<ReviewHelpContent> {
    match level {
        acm_os_domain::ReviewHelpLevel::PrerequisiteNames => {
            let targets = prerequisite_targets(markdown)?;
            Some(ReviewHelpContent {
                title: "Prerequisite names".to_owned(),
                markdown: targets
                    .iter()
                    .map(|target| format!("- {target}"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            })
        }
        acm_os_domain::ReviewHelpLevel::Hints => {
            unique_h2_section(markdown, "Hints")?;
            let headings = collect_headings(markdown);
            let hints = headings
                .iter()
                .enumerate()
                .filter(|(index, heading)| {
                    heading.level == HeadingLevel::H3
                        && direct_parent_h2(&headings, *index).is_some_and(|name| name == "Hints")
                })
                .map(|(index, heading)| {
                    markdown[heading.start..section_end(&headings, index, markdown.len())].trim()
                })
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            (!hints.is_empty()).then(|| ReviewHelpContent {
                title: "Hints".to_owned(),
                markdown: hints.join("\n\n"),
            })
        }
        acm_os_domain::ReviewHelpLevel::PrerequisiteContent => None,
        acm_os_domain::ReviewHelpLevel::OldIdeaOrCode => {
            let sections = ["思路", "代码"]
                .iter()
                .filter_map(|name| unique_h2_section(markdown, name))
                .collect::<Vec<_>>();
            (!sections.is_empty()).then(|| ReviewHelpContent {
                title: "Old idea / code".to_owned(),
                markdown: sections.join("\n\n"),
            })
        }
        acm_os_domain::ReviewHelpLevel::FullSolution => {
            unique_h2_section(markdown, "题解").map(|section| ReviewHelpContent {
                title: "Full solution".to_owned(),
                markdown: section.to_owned(),
            })
        }
    }
}

pub(crate) fn prerequisite_targets(markdown: &str) -> Option<Vec<String>> {
    let section = unique_h2_section(markdown, "前置知识")?;
    let bytes = section.as_bytes();
    let mut targets = Vec::new();
    let mut cursor = 0;
    while cursor + 1 < bytes.len() {
        if bytes[cursor] == b'[' && bytes[cursor + 1] == b'[' {
            let content_start = cursor + 2;
            let Some(relative_end) = section[content_start..].find("]]") else {
                break;
            };
            let raw = &section[content_start..content_start + relative_end];
            let target = raw
                .split('|')
                .next()
                .unwrap_or_default()
                .split('#')
                .next()
                .unwrap_or_default()
                .trim()
                .trim_end_matches(".md");
            if !target.is_empty() && !targets.iter().any(|known| known == target) {
                targets.push(target.to_owned());
            }
            cursor = content_start + relative_end + 2;
        } else {
            cursor += 1;
        }
    }
    (!targets.is_empty()).then_some(targets)
}

fn unique_h2_section<'a>(markdown: &'a str, name: &str) -> Option<&'a str> {
    let headings = collect_headings(markdown);
    let matches = headings
        .iter()
        .enumerate()
        .filter(|(_, heading)| heading.level == HeadingLevel::H2 && heading.name == name)
        .collect::<Vec<_>>();
    let [(index, heading)] = matches.as_slice() else {
        return None;
    };
    let end = section_end(&headings, *index, markdown.len());
    let body = markdown[heading.heading_end..end].trim();
    (!body.is_empty()).then(|| markdown[heading.start..end].trim())
}

pub(crate) fn section_contains_wikilink_item(
    markdown: &str,
    start: usize,
    end: usize,
    target: &str,
) -> bool {
    let expected = format!("[[{target}]]");
    let mut item_text: Option<String> = None;
    for (event, _) in Parser::new(&markdown[start..end]).into_offset_iter() {
        match event {
            Event::Start(Tag::Item) => item_text = Some(String::new()),
            Event::Text(text) => {
                if let Some(value) = &mut item_text {
                    value.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(value) = &mut item_text {
                    value.push(' ');
                }
            }
            Event::End(TagEnd::Item) => {
                if item_text
                    .take()
                    .is_some_and(|value| value.trim() == expected)
                {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        parse_problem_markdown, prerequisite_targets, review_help_content,
        section_contains_wikilink_item,
    };
    use acm_os_application::MarkdownParseWarning;

    #[test]
    fn parses_known_sections_and_only_direct_solution_routes() {
        let markdown = "# Problem\n\n## 前置知识\n\n## 题解\n\n### 双指针 ×\nbody\n\n#### 证明\nbody\n\n### 动态规划\n\n## Hints\n\n### Hint 1\n\n## 额外题目\n";
        let projection = parse_problem_markdown(markdown, "digest".to_owned());

        assert_eq!(
            projection
                .known_sections
                .iter()
                .map(|section| section.name.as_str())
                .collect::<Vec<_>>(),
            ["前置知识", "题解", "Hints", "额外题目"]
        );
        assert_eq!(
            projection
                .solution_routes
                .iter()
                .map(|route| route.name.as_str())
                .collect::<Vec<_>>(),
            ["双指针 ×", "动态规划"]
        );
        assert!(projection.warnings.is_empty());
        assert_eq!(
            projection.solution_routes[0].content_markdown,
            "body\n\n#### 证明\nbody"
        );
        assert!(projection.solution_routes[0]
            .content_html
            .contains("<h4>证明</h4>"));
        assert_eq!(projection.solution_routes[1].content_markdown, "");
    }

    #[test]
    fn route_content_preserves_math_code_and_unicode_boundaries() {
        let projection = parse_problem_markdown("# 题目\n## 题解\n### 标准推导\n公式 $a_i+b_j$\n\n$$x^2$$\n\n```cpp\n// $code$\n```\n### 另一个方法\n不同正文\n## 额外题目\n不可混入", "digest".to_owned());
        let route = &projection.solution_routes[0];
        assert!(route.content_html.contains("math-inline\">a_i+b_j"));
        assert!(route.content_html.contains("math-display\">x^2"));
        assert!(route.content_html.contains("// $code$"));
        assert!(!route.content_markdown.contains("不同正文"));
        assert_eq!(projection.solution_routes[1].content_markdown, "不同正文");
    }

    #[test]
    fn document_projection_preserves_order_introduction_and_prompt() {
        let markdown = "---\nproblem_id: 42\n---\n# Problem\n这是完整题意 $n^2$。\n\n## 前置知识\n- 二分图\n## Hints\n### Hint 1\n尝试染色\n## 题解\n方法总览\n### 标准推导\n正确性证明\n## 额外题目\n暂无\n## Prompt\n我的原始提示词\n## 自定义章节\n保留正文\n";
        let projection = parse_problem_markdown(markdown, "digest".to_owned());
        assert!(projection.introduction_html.contains("这是完整题意"));
        assert!(!projection.introduction_html.contains("problem_id"));
        assert!(!projection.introduction_html.contains("Problem"));
        assert_eq!(projection.document_sections.iter().map(|section| section.name.as_str()).collect::<Vec<_>>(), ["前置知识", "Hints", "题解", "额外题目", "Prompt", "自定义章节"]);
        assert!(projection.document_sections[1].content_html.contains("尝试染色"));
        assert!(projection.document_sections[2].content_html.contains("方法总览"));
        assert!(!projection.document_sections[2].content_html.contains("正确性证明"));
        assert_eq!(projection.document_sections[4].content_markdown, "我的原始提示词");
        assert_eq!(projection.solution_routes[0].content_markdown, "正确性证明");
        let no_title = parse_problem_markdown("---\r\nkey: value\r\n---\r\n题意正文\r\n## Hints\r\n提示", "digest".to_owned());
        assert!(no_title.introduction_html.contains("题意正文"));
        assert!(!no_title.introduction_html.contains("key:"));
    }

    #[test]
    fn warns_for_duplicate_known_sections_without_guessing() {
        let markdown = "## 题解\n### First\n## 题解\n### Second\n";
        let projection = parse_problem_markdown(markdown, "digest".to_owned());

        assert_eq!(projection.known_sections.len(), 2);
        assert_eq!(projection.solution_routes.len(), 2);
        assert_eq!(
            projection.warnings,
            [MarkdownParseWarning::DuplicateKnownSection {
                name: "题解".to_owned(),
                count: 2,
            }]
        );
    }

    #[test]
    fn does_not_attach_a_route_across_a_new_top_level_section() {
        let markdown = "## 题解\n### Route\n# Another document\n### Not a route\n";
        let projection = parse_problem_markdown(markdown, "digest".to_owned());

        assert_eq!(projection.solution_routes.len(), 1);
        assert_eq!(projection.solution_routes[0].name, "Route");
    }

    #[test]
    fn wikilink_item_detection_uses_markdown_list_structure() {
        let markdown = "## 额外题目\n- [[CF-2000-A]]\n\n`- [[CF-2000-B]]`\n";
        assert!(section_contains_wikilink_item(
            markdown,
            0,
            markdown.len(),
            "CF-2000-A",
        ));
        assert!(!section_contains_wikilink_item(
            markdown,
            0,
            markdown.len(),
            "CF-2000-B",
        ));
    }

    #[test]
    fn resolves_only_explicit_review_help_sections() {
        let markdown = "# P\n\n## 前置知识\n- [[Graphs#DFS|Graph traversal]]\n\n## Hints\n### Hint 1\nTry a stack.\n\n### Hint 2\nTrack colors.\n\n## 思路\nSecret idea.\n\n## 代码\n```cpp\nsolve();\n```\n\n## 题解\nComplete proof.\n";
        assert_eq!(
            prerequisite_targets(markdown),
            Some(vec!["Graphs".to_owned()])
        );
        let names =
            review_help_content(markdown, acm_os_domain::ReviewHelpLevel::PrerequisiteNames)
                .expect("prerequisite names");
        assert_eq!(names.markdown, "- Graphs");
        let hints =
            review_help_content(markdown, acm_os_domain::ReviewHelpLevel::Hints).expect("hints");
        assert!(hints.markdown.contains("### Hint 1"));
        assert!(hints.markdown.contains("### Hint 2"));
        let old = review_help_content(markdown, acm_os_domain::ReviewHelpLevel::OldIdeaOrCode)
            .expect("old idea/code");
        assert!(old.markdown.contains("Secret idea."));
        assert!(old.markdown.contains("solve();"));
        let solution = review_help_content(markdown, acm_os_domain::ReviewHelpLevel::FullSolution)
            .expect("solution");
        assert!(solution.markdown.contains("Complete proof."));
    }

    #[test]
    fn duplicate_or_empty_sections_are_not_revealable() {
        let markdown = "## Hints\n\n## Hints\n### One\nDo this\n\n## 题解\n";
        assert!(review_help_content(markdown, acm_os_domain::ReviewHelpLevel::Hints).is_none());
        assert!(
            review_help_content(markdown, acm_os_domain::ReviewHelpLevel::FullSolution).is_none()
        );
    }
}
