#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encrypted data could not be authenticated")]
    AuthenticationFailed,
    #[error("key derivation failed")]
    KeyDerivationFailed,
    #[error("unsupported encryption envelope version: {0}")]
    UnsupportedVersion(u8),
}
