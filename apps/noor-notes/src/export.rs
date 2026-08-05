use noor_domain::{ListKind, Note, TextMarks};

pub fn export_plain(note: &Note) -> String {
    note.content.clone()
}

pub fn export_markdown(note: &Note) -> String {
    let Some(document) = note
        .rich_content
        .as_ref()
        .filter(|document| document.is_supported())
    else {
        return format!("{}\n", note.content);
    };
    let mut output = String::new();
    for (index, block) in document.blocks.iter().enumerate() {
        match block.list {
            Some(ListKind::Bullet) => output.push_str("- "),
            Some(ListKind::Numbered) => output.push_str(&format!("{}. ", index + 1)),
            None => {}
        }
        for span in &block.spans {
            output.push_str(&markdown_span(&span.text, &span.marks));
        }
        output.push('\n');
    }
    output
}

fn markdown_span(text: &str, marks: &TextMarks) -> String {
    let mut value = text.to_string();
    if marks.strikethrough {
        value = format!("~~{value}~~");
    }
    if marks.underline {
        value = format!("<u>{value}</u>");
    }
    if marks.italic {
        value = format!("*{value}*");
    }
    if marks.bold {
        value = format!("**{value}**");
    }
    value
}
