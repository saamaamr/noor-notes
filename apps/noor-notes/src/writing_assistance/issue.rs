use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IssueSource {
    OfflineGrammar,
    CloudGrammar,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssistanceIssue {
    pub range: Range<usize>,
    pub category: String,
    pub message: String,
    pub replacements: Vec<String>,
    pub source: IssueSource,
}
