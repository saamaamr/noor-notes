use noor_notes::writing_assistance::{CheckRegion, GrammarService};

#[test]
fn english_grammar_returns_character_indexed_replacements() {
    let service = GrammarService::default();
    let text = "This is an test.";

    let issues = service.check(
        text,
        "en-US",
        &[CheckRegion {
            start: 0,
            end: text.chars().count(),
        }],
    );

    assert!(issues.iter().any(|issue| {
        issue.range.start < issue.range.end
            && issue.range.end <= text.chars().count()
            && issue
                .replacements
                .iter()
                .any(|replacement| replacement == "a")
    }));
}

#[test]
fn grammar_offsets_are_shifted_by_unicode_characters_not_bytes() {
    let service = GrammarService::default();
    let prefix = "বাংলা ";
    let text = format!("{prefix}This is an test.");
    let start = prefix.chars().count();

    let issues = service.check(
        &text,
        "en",
        &[CheckRegion {
            start,
            end: text.chars().count(),
        }],
    );

    assert!(!issues.is_empty());
    assert!(issues.iter().all(|issue| issue.range.start >= start));
    assert!(
        issues
            .iter()
            .all(|issue| issue.range.end <= text.chars().count())
    );
}

#[test]
fn unsupported_languages_and_excluded_ranges_return_no_findings() {
    let service = GrammarService::default();
    let text = "This is an test.";
    let whole_text = [CheckRegion {
        start: 0,
        end: text.chars().count(),
    }];

    assert!(service.check(text, "bn", &whole_text).is_empty());
    assert!(service.check(text, "en", &[]).is_empty());
    assert!(
        service
            .check(
                text,
                "en",
                &[CheckRegion {
                    start: 5,
                    end: text.chars().count() + 1,
                }],
            )
            .is_empty()
    );
}
