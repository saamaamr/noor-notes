mod cloud;
mod controller;
mod grammar;
mod issue;
mod issue_popover;
mod prediction;
mod preferences;
mod scope;
mod spelling;

pub use cloud::{CloudAssistanceClient, CloudError, TextScope, paragraph_scope, sentence_scope};
pub use controller::{AssistanceStatus, WritingAssistanceController};
pub use grammar::GrammarService;
pub use issue::{AssistanceIssue, IssueSource};
pub use issue_popover::IssuePopover;
pub use prediction::PredictionModel;
pub use preferences::{
    PreferenceError, ProviderConfiguration, ResolvedWritingAssistance,
    WritingAssistancePreferences, WritingAssistanceStore, provider_requires_api_key,
    validate_provider_endpoint,
};
pub use scope::{CheckRegion, checkable_regions, plain_text_regions};
pub use spelling::{SpellLanguage, SpellService, SpellSession};
