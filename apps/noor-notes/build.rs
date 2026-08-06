fn main() {
    glib_build_tools::compile_resources(
        &["resources"],
        "resources/noor-notes.gresource.xml",
        "noor-notes.gresource",
    );
}
