use noor_domain::{EditorMode, SourceLanguage};
use noor_notes::appearance::EffectiveTheme;
use noor_notes::editor::SourceEditorAdapter;
use noor_notes::writing_assistance::{CheckRegion, checkable_regions, plain_text_regions};
use sourceview5::prelude::*;

#[test]
fn scope_respects_unicode_markdown_exclusions_and_code_contexts() {
    gtk::init().unwrap();
    character_regions_do_not_split_bengali_text();
    markdown_excludes_inline_and_fenced_code();
    code_includes_only_comments_and_strings();
}

fn character_regions_do_not_split_bengali_text() {
    let text = "আমি লিখি";

    let regions = plain_text_regions(text);

    assert_eq!(
        regions,
        vec![CheckRegion {
            start: 0,
            end: text.chars().count(),
        }]
    );
}

fn markdown_excludes_inline_and_fenced_code() {
    let text = "Useful prose `let hidden = true` remains.\n```rust\nsecret_call();\n```";
    let editor = SourceEditorAdapter::new_with_theme(
        text,
        Some(&SourceLanguage::Markdown),
        EffectiveTheme::Snow,
    );
    editor
        .buffer()
        .ensure_highlight(&editor.buffer().start_iter(), &editor.buffer().end_iter());

    let included = included_text(
        text,
        &checkable_regions(editor.buffer(), EditorMode::Markdown),
    );

    assert!(included.contains("Useful prose"));
    assert!(included.contains("remains"));
    assert!(!included.contains("hidden"));
    assert!(!included.contains("secret_call"));
}

fn code_includes_only_comments_and_strings() {
    let text = "let executable_identifier = 1;\nlet message = \"support people\"; // clear systems";
    let rust = SourceLanguage::new("rust").unwrap();
    let editor = SourceEditorAdapter::new_with_theme(text, Some(&rust), EffectiveTheme::Snow);
    editor
        .buffer()
        .ensure_highlight(&editor.buffer().start_iter(), &editor.buffer().end_iter());

    let included = included_text(text, &checkable_regions(editor.buffer(), EditorMode::Code));

    assert!(included.contains("support people"));
    assert!(included.contains("clear systems"));
    assert!(!included.contains("executable_identifier"));
    assert!(!included.contains("message"));
}

fn included_text(text: &str, regions: &[CheckRegion]) -> String {
    let characters = text.chars().collect::<Vec<_>>();
    regions
        .iter()
        .map(|region| {
            characters[region.start..region.end]
                .iter()
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("|")
}
