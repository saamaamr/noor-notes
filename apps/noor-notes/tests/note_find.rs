use noor_notes::note_find::{FindOptions, FindResults, replace_all, replace_current};

#[test]
fn unicode_find_counts_char_offsets_and_wraps_navigation() {
    let mut results = FindResults::new("Alpha বাংলা alpha", "ALPHA");
    assert_eq!(results.ranges(), &[(0, 5), (12, 17)]);
    assert_eq!(results.position(), Some((1, 2)));
    results.next();
    assert_eq!(results.position(), Some((2, 2)));
    results.next();
    assert_eq!(results.position(), Some((1, 2)));
    results.previous();
    assert_eq!(results.position(), Some((2, 2)));
    results.update("বাংলা", "missing");
    assert!(results.ranges().is_empty());
    assert_eq!(results.position(), None);
}

#[test]
fn find_supports_match_case_and_whole_words() {
    let sensitive = FindResults::with_options(
        "Note notebook note NOTE",
        "Note",
        FindOptions {
            match_case: true,
            whole_word: true,
        },
    );
    assert_eq!(sensitive.ranges(), &[(0, 4)]);

    let insensitive = FindResults::with_options(
        "Note notebook note NOTE",
        "note",
        FindOptions {
            match_case: false,
            whole_word: true,
        },
    );
    assert_eq!(insensitive.ranges(), &[(0, 4), (14, 18), (19, 23)]);
}

#[test]
fn replace_current_and_all_preserve_unicode_offsets() {
    let results = FindResults::new("বাংলা note বাংলা", "বাংলা");
    let (text, cursor) = replace_current("বাংলা note বাংলা", &results, "নূর").unwrap();
    assert_eq!(text, "নূর note বাংলা");
    assert_eq!(cursor, 3);

    let (text, replacements) = replace_all(
        "One one someone ONE",
        "one",
        "note",
        FindOptions {
            match_case: false,
            whole_word: true,
        },
    );
    assert_eq!(text, "note note someone note");
    assert_eq!(replacements, 3);
}
