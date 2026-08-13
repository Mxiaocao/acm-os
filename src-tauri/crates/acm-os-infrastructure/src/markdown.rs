use std::collections::BTreeMap;

use acm_os_application::{
    KnownMarkdownSection, MarkdownParseWarning, ProblemMarkdownProjection, SolutionRoute,
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
    let mut counts = BTreeMap::<String, usize>::new();

    for (index, heading) in headings.iter().enumerate() {
        let end_offset = section_end(&headings, index, markdown.len());
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
            solution_routes.push(SolutionRoute {
                name: heading.name.clone(),
                start_offset: heading.start,
                end_offset,
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
        known_sections,
        solution_routes,
        warnings,
    }
}

fn collect_headings(markdown: &str) -> Vec<Heading> {
    let mut headings = Vec::new();
    let mut current: Option<Heading> = None;
    for (event, range) in Parser::new(markdown).into_offset_iter() {
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
