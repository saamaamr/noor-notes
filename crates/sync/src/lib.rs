//! Encrypted offline-first synchronization.

mod client;
mod types;

pub use client::{SupabaseClient, SyncClientError};
pub use types::{AuthSession, RemoteRevision};
