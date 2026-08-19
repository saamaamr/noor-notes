use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

#[test]
fn local_installer_creates_a_distinct_dev_app_without_touching_notes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    let bin_root = root.join("bin");
    let data_root = root.join("data");
    let target_root = root.join("target");
    let fake_bin = root.join("fake-bin");
    fs::create_dir_all(&bin_root).unwrap();
    fs::create_dir_all(data_root.join("applications")).unwrap();
    fs::create_dir_all(data_root.join("noor-notes")).unwrap();
    fs::create_dir_all(target_root.join("release")).unwrap();
    fs::create_dir_all(&fake_bin).unwrap();

    fs::write(bin_root.join("noor-notes"), b"legacy-local-binary").unwrap();
    fs::write(
        data_root
            .join("applications")
            .join("io.github.saamaamr.NoorNotes.desktop"),
        b"legacy-local-launcher",
    )
    .unwrap();
    fs::create_dir_all(data_root.join("metainfo")).unwrap();
    fs::write(
        data_root
            .join("metainfo")
            .join("io.github.saamaamr.NoorNotes.metainfo.xml"),
        b"legacy-local-metainfo",
    )
    .unwrap();
    fs::write(
        data_root.join("noor-notes").join("notes.db"),
        b"preserve-current-local-notes",
    )
    .unwrap();
    fs::write(
        target_root.join("release").join("noor-notes"),
        b"new-development-binary",
    )
    .unwrap();

    let cargo_log = root.join("cargo-args");
    write_executable(
        &fake_bin.join("cargo"),
        "#!/bin/sh\nprintf '%s\\n' \"$*\" > \"$CARGO_LOG\"\n",
    );
    write_executable(&fake_bin.join("gnome-extensions"), "#!/bin/sh\nexit 0\n");
    write_executable(
        &fake_bin.join("update-desktop-database"),
        "#!/bin/sh\nexit 0\n",
    );

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let system_path = std::env::var("PATH").unwrap_or_default();
    let output = Command::new("bash")
        .arg(repo_root.join("scripts/install-local.sh"))
        .env("HOME", root)
        .env("XDG_BIN_HOME", &bin_root)
        .env("XDG_DATA_HOME", &data_root)
        .env("CARGO_TARGET_DIR", &target_root)
        .env("CARGO_LOG", &cargo_log)
        .env("PATH", format!("{}:{system_path}", fake_bin.display()))
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(bin_root.join("noor-notes-dev")).unwrap(),
        b"new-development-binary"
    );
    assert!(!bin_root.join("noor-notes").exists());
    assert!(
        !data_root
            .join("applications")
            .join("io.github.saamaamr.NoorNotes.desktop")
            .exists()
    );
    assert!(
        !data_root
            .join("metainfo")
            .join("io.github.saamaamr.NoorNotes.metainfo.xml")
            .exists()
    );

    let launcher = fs::read_to_string(
        data_root
            .join("applications")
            .join("io.github.saamaamr.NoorNotes.Devel.desktop"),
    )
    .unwrap();
    assert!(launcher.contains("Name=Noor Notes Dev"));
    assert!(launcher.contains("Exec=noor-notes-dev"));
    assert!(launcher.contains("StartupWMClass=io.github.saamaamr.NoorNotes.Devel"));
    assert!(
        fs::read_to_string(&cargo_log)
            .unwrap()
            .contains("--features development")
    );
    assert_eq!(
        fs::read(data_root.join("noor-notes").join("notes.db")).unwrap(),
        b"preserve-current-local-notes"
    );
}

fn write_executable(path: &Path, contents: &str) {
    fs::write(path, contents).unwrap();
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}
