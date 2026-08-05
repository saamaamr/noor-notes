#[cfg(unix)]
#[test]
fn scan_rejects_symlinked_info_and_oversized_metadata() {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("real-info"),
        "content content-1\nwidth 400\nheight 300\n",
    )
    .unwrap();
    symlink(dir.path().join("real-info"), dir.path().join("info-link")).unwrap();
    let preview = noor_xpad_import::scan_xpad(dir.path()).unwrap();
    assert_eq!(preview.importable.len(), 0);
    assert_eq!(preview.skipped.len(), 1);
    std::fs::remove_file(dir.path().join("info-link")).unwrap();
    std::fs::write(dir.path().join("info-large"), vec![b'x'; 65 * 1024]).unwrap();
    let preview = noor_xpad_import::scan_xpad(dir.path()).unwrap();
    assert_eq!(preview.skipped.len(), 1);
}
