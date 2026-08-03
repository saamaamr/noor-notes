use noor_notes::vault_setup::{OnboardingState, VaultOnboarding};

#[test]
fn sync_cannot_enable_before_recovery_confirmation() {
    let mut onboarding = VaultOnboarding::new();
    onboarding.account_authenticated();
    let recovery = onboarding.begin_vault(b"strong local passphrase").unwrap();
    assert!(recovery.contains('-'));
    assert_eq!(onboarding.state(), OnboardingState::RecoveryKeyRequired);
    assert!(!onboarding.sync_enabled());

    onboarding.cancel();

    assert_eq!(onboarding.state(), OnboardingState::AccountReady);
    assert!(!onboarding.sync_enabled());
}

#[test]
fn confirmed_recovery_key_enables_sync() {
    let mut onboarding = VaultOnboarding::new();
    onboarding.account_authenticated();
    onboarding.begin_vault(b"strong local passphrase").unwrap();

    onboarding.confirm_recovery().unwrap();

    assert_eq!(onboarding.state(), OnboardingState::Ready);
    assert!(onboarding.sync_enabled());
}
