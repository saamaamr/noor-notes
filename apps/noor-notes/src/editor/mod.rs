mod adapter;
mod autosave_controller;
mod languages;
mod rich_adapter;
mod search;
mod session;
mod source_adapter;
mod statistics;

pub use adapter::{AdapterCapabilities, EditorAdapter};
pub use autosave_controller::{AutosaveController, SaveGeneration, SavePhase};
pub use languages::{available_language_ids, resolve_language};
pub use rich_adapter::RichEditorAdapter;
pub use search::SearchOptions;
pub use session::EditorSession;
pub use source_adapter::SourceEditorAdapter;
pub use statistics::EditorStatistics;
