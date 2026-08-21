use gtk::prelude::*;
use noor_notes::appearance::EffectiveTheme;
use noor_notes::editor::SourceEditorAdapter;
use noor_notes::writing_assistance::SpellService;

#[test]
fn spelling_attaches_named_actions_lists_languages_and_can_be_toggled() {
    gtk::init().unwrap();
    let editor = SourceEditorAdapter::new_rich("mispelled", EffectiveTheme::Snow);

    let session = SpellService::attach(editor.buffer(), editor.view(), "en_US", true);

    assert!(session.is_available());
    assert!(session.is_enabled());
    let menu = editor.view().extra_menu();
    let label = menu
        .item_attribute_value(0, "label", None)
        .and_then(|value| value.get::<String>());
    assert_eq!(label.as_deref(), Some("Spelling"));

    let languages = SpellService::installed_languages();
    let english = languages
        .iter()
        .find(|language| language.code == "en_US")
        .expect("en_US dictionary should be installed for the test environment");
    assert_eq!(english.name, "English (United States)");
    assert!(
        languages
            .windows(2)
            .all(|pair| pair[0].name <= pair[1].name)
    );
    assert!(
        languages
            .windows(2)
            .all(|pair| pair[0].code != pair[1].code)
    );

    session.set_enabled(false);
    assert!(!session.is_enabled());
    session.set_enabled(true);
    assert!(session.is_enabled());

    session.set_language("definitely_not_installed");
    assert!(!session.is_available());
    assert!(!session.is_enabled());
}
