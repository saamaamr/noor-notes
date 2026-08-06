mod adapter;
mod autosave_controller;
mod search;
mod session;
mod statistics;

pub use adapter::{AdapterCapabilities, EditorAdapter};
pub use autosave_controller::{AutosaveController, SaveGeneration, SavePhase};
pub use search::SearchOptions;
pub use session::EditorSession;
pub use statistics::EditorStatistics;
