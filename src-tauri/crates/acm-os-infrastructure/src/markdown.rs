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
                if item_text.take().is_some_and(|value| value.trim() == expected) {
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
    use super::{parse_problem_markdown, section_contains_wikilink_item};
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
}
