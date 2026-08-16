use std::sync::Mutex;

use harper_core::linting::{LintGroup, LintKind, Linter, Suggestion};
use harper_core::parsers::PlainEnglish;
use harper_core::spell::FstDictionary;
use harper_core::{Dialect, Document};

use super::{AssistanceIssue, CheckRegion, IssueSource};

pub struct GrammarService {
    linter: Mutex<LintGroup>,
}

impl Default for GrammarService {
    fn default() -> Self {
        Self {
            linter: Mutex::new(LintGroup::new_curated(
                FstDictionary::curated(),
                Dialect::American,
            )),
        }
    }
}

impl GrammarService {
    pub fn check(
        &self,
        text: &str,
        language: &str,
        regions: &[CheckRegion],
    ) -> Vec<AssistanceIssue> {
        if !is_english(language) || regions.is_empty() {
            return Vec::new();
        }
        let characters = text.chars().collect::<Vec<_>>();
        if regions
            .iter()
            .any(|region| region.start >= region.end || region.end > characters.len())
        {
            return Vec::new();
        }
        let mut linter = self.linter.lock().expect("grammar linter mutex poisoned");
        let parser = PlainEnglish;
        let mut issues = Vec::new();

        for region in regions {
            let source = characters[region.start..region.end]
                .iter()
                .collect::<String>();
            let document = Document::new_curated(&source, &parser);
            for lint in linter
                .lint(&document)
                .into_iter()
                .filter(|lint| lint.lint_kind != LintKind::Spelling)
            {
                if lint.span.start > lint.span.end || lint.span.end > source.chars().count() {
                    continue;
                }
                let original = source
                    .chars()
                    .skip(lint.span.start)
                    .take(lint.span.end - lint.span.start)
                    .collect::<String>();
                let mut replacements = lint
                    .suggestions
                    .iter()
                    .filter_map(|suggestion| replacement_for(suggestion, &original))
                    .filter(|replacement| {
                        replacement.chars().count() <= 256
                            && !replacement.chars().any(char::is_control)
                    })
                    .collect::<Vec<_>>();
                replacements.dedup();
                replacements.truncate(5);
                issues.push(AssistanceIssue {
                    range: (region.start + lint.span.start)..(region.start + lint.span.end),
                    category: lint.lint_kind.to_string(),
                    message: lint.message,
                    replacements,
                    source: IssueSource::OfflineGrammar,
                });
            }
        }
        issues
    }
}

fn replacement_for(suggestion: &Suggestion, original: &str) -> Option<String> {
    match suggestion {
        Suggestion::ReplaceWith(characters) => Some(characters.iter().collect()),
        Suggestion::InsertAfter(characters) => Some(format!(
            "{original}{}",
            characters.iter().collect::<String>()
        )),
        Suggestion::Remove => Some(String::new()),
    }
}

fn is_english(language: &str) -> bool {
    let language = language.trim().to_ascii_lowercase();
    language == "en" || language.starts_with("en-") || language.starts_with("en_")
}
