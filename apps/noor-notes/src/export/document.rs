use noor_domain::{Alignment, EditorMode, ListKind, Note, TextMarks};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportDocument {
    pub title: String,
    pub blocks: Vec<ExportBlock>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportBlock {
    pub kind: ExportBlockKind,
    pub alignment: Alignment,
    pub runs: Vec<ExportRun>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExportBlockKind {
    Paragraph,
    ListItem { kind: ListKind, ordinal: u32 },
    CodeBlock,
    MarkdownSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRun {
    pub text: String,
    pub marks: TextMarks,
}

impl ExportDocument {
    pub fn from_note(note: &Note) -> Self {
        let title = note.display_title().to_string();
        let blocks = match note.editor_mode {
            EditorMode::Rich => rich_blocks(note),
            EditorMode::Markdown => {
                vec![source_block(ExportBlockKind::MarkdownSource, &note.content)]
            }
            EditorMode::Code => vec![source_block(ExportBlockKind::CodeBlock, &note.content)],
            EditorMode::PlainText => plain_blocks(&note.content),
        };
        Self { title, blocks }
    }
}

fn rich_blocks(note: &Note) -> Vec<ExportBlock> {
    let Some(document) = note
        .rich_content
        .as_ref()
        .filter(|document| document.is_supported())
    else {
        return plain_blocks(&note.content);
    };

    let mut numbered_ordinal = 0_u32;
    document
        .blocks
        .iter()
        .map(|block| {
            let kind = match block.list {
                Some(ListKind::Bullet) => {
                    numbered_ordinal = 0;
                    ExportBlockKind::ListItem {
                        kind: ListKind::Bullet,
                        ordinal: 1,
                    }
                }
                Some(ListKind::Numbered) => {
                    numbered_ordinal += 1;
                    ExportBlockKind::ListItem {
                        kind: ListKind::Numbered,
                        ordinal: numbered_ordinal,
                    }
                }
                None => {
                    numbered_ordinal = 0;
                    ExportBlockKind::Paragraph
                }
            };
            ExportBlock {
                kind,
                alignment: block.alignment,
                runs: block
                    .spans
                    .iter()
                    .map(|span| ExportRun {
                        text: span.text.clone(),
                        marks: span.marks.clone(),
                    })
                    .collect(),
            }
        })
        .collect()
}

fn plain_blocks(content: &str) -> Vec<ExportBlock> {
    content
        .split('\n')
        .map(|line| source_block(ExportBlockKind::Paragraph, line))
        .collect()
}

fn source_block(kind: ExportBlockKind, text: &str) -> ExportBlock {
    ExportBlock {
        kind,
        alignment: Alignment::Start,
        runs: vec![ExportRun {
            text: text.to_string(),
            marks: TextMarks::default(),
        }],
    }
}
