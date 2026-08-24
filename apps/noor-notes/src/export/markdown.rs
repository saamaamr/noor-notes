use noor_domain::{ListKind, TextMarks};

use super::{ExportBlockKind, ExportDocument};

pub fn render(document: &ExportDocument) -> String {
    let mut output = format!("# {}\n\n", document.title);
    for block in &document.blocks {
        match block.kind {
            ExportBlockKind::ListItem {
                kind: ListKind::Bullet,
                ..
            } => output.push_str("- "),
            ExportBlockKind::ListItem {
                kind: ListKind::Numbered,
                ordinal,
            } => output.push_str(&format!("{ordinal}. ")),
            ExportBlockKind::CodeBlock => output.push_str("```\n"),
            ExportBlockKind::Paragraph | ExportBlockKind::MarkdownSource => {}
        }
        for run in &block.runs {
            output.push_str(&render_run(&run.text, &run.marks));
        }
        if block.kind == ExportBlockKind::CodeBlock {
            output.push_str("\n```\n");
        } else {
            output.push('\n');
        }
    }
    output
}

fn render_run(text: &str, marks: &TextMarks) -> String {
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
