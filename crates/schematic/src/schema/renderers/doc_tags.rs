//! Tags written into doc comments, such as `@since 1.2.0`, which the renderers
//! with a place for them pull out of the description.

/// A tag on its own line of a doc comment, in the style of JSDoc, such as
/// `@since 1.2.0`.
#[derive(Clone, Debug, PartialEq)]
pub struct DocTag {
    /// The name, without the `@`, as written.
    pub name: String,

    /// The text after the name, if any.
    pub value: Option<String>,
}

#[cfg(any(feature = "renderer_pkl_schema", feature = "renderer_typescript"))]
impl DocTag {
    /// Render the tag as it would be written, such as `@since 1.2.0`.
    pub fn to_line(&self) -> String {
        match &self.value {
            Some(value) => format!("@{} {value}", self.name),
            None => format!("@{}", self.name),
        }
    }
}

/// Split the tags out of a doc comment, returning the comment without them,
/// and the tags in the order they were written. A tag is a line that begins
/// with `@` and a name, and lasts for that line only. A line within a fenced
/// code block is never a tag, as an example may begin one with a decorator.
pub fn split_doc_tags(comment: &str) -> (String, Vec<DocTag>) {
    let mut lines: Vec<&str> = vec![];
    let mut tags = vec![];
    let mut fence: Option<&str> = None;
    let mut removed = false;

    for line in comment.lines() {
        let trimmed = line.trim();

        if let Some(marker) = ["```", "~~~"]
            .into_iter()
            .find(|marker| trimmed.starts_with(marker))
        {
            match fence {
                Some(open) if open == marker => fence = None,
                None => fence = Some(marker),
                _ => {}
            };
        } else if fence.is_none()
            && let Some(tag) = parse_tag(trimmed)
        {
            tags.push(tag);
            removed = true;
            continue;
        }

        // A tag removed from between two paragraphs leaves both of their
        // blank lines behind, where only one is needed
        if trimmed.is_empty() && removed && lines.last().is_some_and(|last| last.trim().is_empty())
        {
            continue;
        }

        removed = false;
        lines.push(line);
    }

    (lines.join("\n").trim().to_owned(), tags)
}

fn parse_tag(line: &str) -> Option<DocTag> {
    let rest = line.strip_prefix('@')?;
    let (name, value) = rest.split_at(rest.find(char::is_whitespace).unwrap_or(rest.len()));
    let mut chars = name.chars();

    // A name that continues with anything else, such as the `/` of a scoped
    // package, is text that happens to start with `@`
    let is_name = chars.next().is_some_and(|ch| ch.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');

    if !is_name {
        return None;
    }

    let value = value.trim();

    Some(DocTag {
        name: name.to_owned(),
        value: (!value.is_empty()).then(|| value.to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tag(name: &str, value: Option<&str>) -> DocTag {
        DocTag {
            name: name.into(),
            value: value.map(|value| value.into()),
        }
    }

    #[test]
    fn splits_tags_from_the_description() {
        assert_eq!(
            split_doc_tags("Enables the cache.\n@since 1.2.0\n@experimental"),
            (
                "Enables the cache.".into(),
                vec![tag("since", Some("1.2.0")), tag("experimental", None)]
            )
        );
    }

    #[test]
    fn keeps_one_blank_line_between_paragraphs() {
        assert_eq!(
            split_doc_tags("First.\n\n@since 1.2.0\n\nSecond."),
            (
                "First.\n\nSecond.".into(),
                vec![tag("since", Some("1.2.0"))]
            )
        );
    }

    #[test]
    fn ignores_text_that_starts_with_an_at() {
        let comment = "Uses\n@scope/package\n@ alone\n`@since` in code";

        assert_eq!(split_doc_tags(comment), (comment.into(), vec![]));
    }

    #[test]
    fn ignores_lines_in_code_blocks() {
        let comment = "Example:\n\n```py\n@decorator\n~~~\n```\n\n~~~\n@other\n~~~";

        assert_eq!(split_doc_tags(comment), (comment.into(), vec![]));
    }
}
