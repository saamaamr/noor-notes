use noor_domain::ListKind;

use super::{ExportBlockKind, ExportDocument};

pub fn render(document: &ExportDocument) -> String {
    let mut output = String::new();
    output.push_str(&document.title);
    output.push_str("\n\n");
    for block in &document.blocks {
        match block.kind {
            ExportBlockKind::ListItem {
                kind: ListKind::Bullet,
                ..
            } => output.push_str("• "),
            ExportBlockKind::ListItem {
                kind: ListKind::Numbered,
                ordinal,
            } => output.push_str(&format!("{ordinal}. ")),
            ExportBlockKind::Paragraph
            | ExportBlockKind::CodeBlock
            | ExportBlockKind::MarkdownSource => {}
        }
        for run in &block.runs {
            output.push_str(&run.text);
        }
        output.push('\n');
    }
    output
}
