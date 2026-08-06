mod adapter;
mod autosave_controller;
mod rich_adapter;
mod search;
mod session;
mod statistics;

pub use adapter::{AdapterCapabilities, EditorAdapter};
pub use autosave_controller::{AutosaveController, SaveGeneration, SavePhase};
pub use rich_adapter::RichEditorAdapter;
pub use search::SearchOptions;
pub use session::EditorSession;
pub use statistics::EditorStatistics;
