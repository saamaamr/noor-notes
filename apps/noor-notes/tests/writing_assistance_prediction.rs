use noor_notes::writing_assistance::PredictionModel;

#[test]
fn ranks_trigrams_before_bigrams_and_filters_partial_unicode_tokens() {
    let mut model = PredictionModel::default();
    model.train(&["আমি আজ লিখি। আমি আজ পড়ি। clear support works. clear support helps.".into()]);

    assert_eq!(model.suggest("clear support", "h", 3)[0], "helps");
    assert_eq!(model.suggest("আমি আজ", "প", 3)[0], "পড়ি");
}

#[test]
fn suggestions_are_limited_deduplicated_and_case_insensitive() {
    let mut model = PredictionModel::default();
    model.train(&[
        "Clear support Helps. clear support helps. clear support works. clear support grows. clear support matters.".into(),
    ]);

    let suggestions = model.suggest("CLEAR SUPPORT", "", 20);
    assert_eq!(suggestions.len(), 3);
    assert_eq!(suggestions[0], "Helps");
    assert_eq!(
        suggestions
            .iter()
            .filter(|value| value.eq_ignore_ascii_case("helps"))
            .count(),
        1
    );
}

#[test]
fn pruning_is_deterministic_and_never_exceeds_the_bound() {
    let corpus = (0..25_100)
        .map(|index| format!("anchor context{index} candidate{index}"))
        .collect::<Vec<_>>();
    let mut first = PredictionModel::default();
    first.train(&corpus);
    let mut second = first.clone();

    first.prune(50_000);
    second.prune(50_000);

    assert_eq!(first.entry_count(), 50_000);
    assert_eq!(first, second);
}
