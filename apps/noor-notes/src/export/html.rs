use noor_domain::{Alignment, ListKind, TextMarks};

use crate::appearance::EffectiveTheme;
use crate::rich_color::{ColorRole, rendered_color};

use super::{ExportBlockKind, ExportDocument};

pub fn render(document: &ExportDocument) -> String {
    let mut body = String::new();
    let mut open_list = None::<ListKind>;

    for block in &document.blocks {
        let list_kind = match block.kind {
            ExportBlockKind::ListItem { kind, .. } => Some(kind),
            _ => None,
        };
        if open_list != list_kind {
            close_list(&mut body, open_list);
            open_list = list_kind;
            if let Some(kind) = open_list {
                body.push_str(match kind {
                    ListKind::Bullet => "<ul>\n",
                    ListKind::Numbered => "<ol>\n",
                });
            }
        }

        let content = block
            .runs
            .iter()
            .map(|run| render_run(&run.text, &run.marks))
            .collect::<String>();
        let alignment = alignment_style(block.alignment);
        match block.kind {
            ExportBlockKind::Paragraph => {
                body.push_str(&format!("<p{alignment}>{content}</p>\n"));
            }
            ExportBlockKind::ListItem { .. } => {
                body.push_str(&format!("<li{alignment}>{content}</li>\n"));
            }
            ExportBlockKind::CodeBlock => {
                body.push_str("<pre><code>");
                body.push_str(&escape_html(
                    &block
                        .runs
                        .iter()
                        .map(|run| run.text.as_str())
                        .collect::<String>(),
                ));
                body.push_str("</code></pre>\n");
            }
            ExportBlockKind::MarkdownSource => {
                body.push_str("<pre class=\"markdown-source\"><code>");
                body.push_str(&escape_html(
                    &block
                        .runs
                        .iter()
                        .map(|run| run.text.as_str())
                        .collect::<String>(),
                ));
                body.push_str("</code></pre>\n");
            }
        }
    }
    close_list(&mut body, open_list);

    let title = escape_html(&document.title);
    format!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title}</title>\n<style>body{{max-width:820px;margin:48px auto;padding:0 24px;font:16px/1.6 system-ui,sans-serif;color:#1F2937}}h1{{font-size:30px;line-height:1.2}}pre{{white-space:pre-wrap;overflow-wrap:anywhere;background:#F7F8FA;padding:16px;border-radius:8px}}code{{font-family:monospace}}li{{margin:.25em 0}}</style>\n</head>\n<body>\n<h1>{title}</h1>\n{body}</body>\n</html>\n"
    )
}

fn render_run(text: &str, marks: &TextMarks) -> String {
    let mut value = escape_html(text);
    if marks.strikethrough {
        value = format!("<s>{value}</s>");
    }
    if marks.underline {
        value = format!("<u>{value}</u>");
    }
    if marks.italic {
        value = format!("<em>{value}</em>");
    }
    if marks.bold {
        value = format!("<strong>{value}</strong>");
    }

    let mut styles = Vec::new();
    if let Some(size) = marks.font_size.filter(|size| (6..=96).contains(size)) {
        styles.push(format!("font-size: {size}pt"));
    }
    if let Some(color) = marks
        .foreground
        .as_deref()
        .and_then(|value| rendered_color(ColorRole::Foreground, value, EffectiveTheme::Snow))
    {
        styles.push(format!("color: {color}"));
    }
    if let Some(color) = marks
        .highlight
        .as_deref()
        .and_then(|value| rendered_color(ColorRole::Highlight, value, EffectiveTheme::Snow))
    {
        styles.push(format!("background-color: {color}"));
    }
    if styles.is_empty() {
        value
    } else {
        format!("<span style=\"{}\">{value}</span>", styles.join("; "))
    }
}

fn alignment_style(alignment: Alignment) -> &'static str {
    match alignment {
        Alignment::Start => "",
        Alignment::Center => " style=\"text-align: center\"",
        Alignment::End => " style=\"text-align: right\"",
        Alignment::Justify => " style=\"text-align: justify\"",
    }
}

fn close_list(output: &mut String, list: Option<ListKind>) {
    if let Some(kind) = list {
        output.push_str(match kind {
            ListKind::Bullet => "</ul>\n",
            ListKind::Numbered => "</ol>\n",
        });
    }
}

fn escape_html(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            _ => output.push(character),
        }
    }
    output
}
