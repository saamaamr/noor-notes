use noor_domain::SourceLanguage;

pub fn resolve_language(
    manager: &sourceview5::LanguageManager,
    language: &SourceLanguage,
) -> Option<sourceview5::Language> {
    manager.language(language.as_str())
}

pub fn available_language_ids(manager: &sourceview5::LanguageManager) -> Vec<String> {
    let mut ids: Vec<String> = manager
        .language_ids()
        .into_iter()
        .map(|value| value.to_string())
        .collect();
    ids.sort();
    ids
}
