mod preferences;

pub use preferences::{
    PreferenceError, ProviderConfiguration, ResolvedWritingAssistance,
    WritingAssistancePreferences, WritingAssistanceStore, provider_requires_api_key,
    validate_provider_endpoint,
};
