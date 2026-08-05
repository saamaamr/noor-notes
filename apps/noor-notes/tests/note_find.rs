use noor_notes::note_find::FindResults;

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
