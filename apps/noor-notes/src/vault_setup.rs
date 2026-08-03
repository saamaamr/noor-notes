use noor_crypto::{CryptoError, RecoveryKey, RecoveryWrappedVault, Vault, WrappedVault};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OnboardingState {
    #[default]
    SignedOut,
    AccountReady,
    RecoveryKeyRequired,
    Ready,
}

#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error("this onboarding action is not valid in the current state")]
    InvalidState,
    #[error(transparent)]
    Crypto(#[from] CryptoError),
}

pub struct VaultEnrollment {
    pub vault: Vault,
    pub wrapped: WrappedVault,
    pub recovery_wrapped: RecoveryWrappedVault,
}

#[derive(Default)]
pub struct VaultOnboarding {
    state: OnboardingState,
    pending: Option<VaultEnrollment>,
    sync_enabled: bool,
}

impl VaultOnboarding {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn state(&self) -> OnboardingState {
        self.state
    }

    pub fn sync_enabled(&self) -> bool {
        self.sync_enabled
    }

    pub fn account_authenticated(&mut self) {
        self.pending = None;
        self.sync_enabled = false;
        self.state = OnboardingState::AccountReady;
    }

    pub fn begin_vault(&mut self, passphrase: &[u8]) -> Result<String, OnboardingError> {
        if self.state != OnboardingState::AccountReady {
            return Err(OnboardingError::InvalidState);
        }
        let (vault, wrapped) = Vault::create(passphrase)?;
        let recovery = RecoveryKey::generate();
        let encoded = recovery.encode();
        let recovery_wrapped = vault.wrap_for_recovery(&recovery)?;
        self.pending = Some(VaultEnrollment {
            vault,
            wrapped,
            recovery_wrapped,
        });
        self.state = OnboardingState::RecoveryKeyRequired;
        Ok(encoded)
    }

    pub fn confirm_recovery(&mut self) -> Result<(), OnboardingError> {
        if self.state != OnboardingState::RecoveryKeyRequired || self.pending.is_none() {
            return Err(OnboardingError::InvalidState);
        }
        self.sync_enabled = true;
        self.state = OnboardingState::Ready;
        Ok(())
    }

    pub fn cancel(&mut self) {
        self.pending = None;
        self.sync_enabled = false;
        self.state = OnboardingState::AccountReady;
    }

    pub fn take_enrollment(&mut self) -> Option<VaultEnrollment> {
        if self.state == OnboardingState::Ready {
            self.pending.take()
        } else {
            None
        }
    }
}
