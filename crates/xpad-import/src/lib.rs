//! Read-only migration from Xpad data.

mod error;
mod importer;
mod parser;
mod report;

pub use error::ImportError;
pub use importer::import_xpad;
pub use parser::scan_xpad;
pub use report::{ImportIssue, ImportPreview, ImportReport, ImportableNote};
