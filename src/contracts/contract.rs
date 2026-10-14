//! Shared contract interface for Belvedere pipeline obligations.

use crate::certify::{BelvedereCertificationError, BelvedereInvariantCheck};

/// A named Belvedere pipeline obligation.
pub trait BelvedereContract {
    /// Evidence accepted by the contract.
    type Input;
    /// Evidence produced by the contract.
    type Output;
    /// Audit artifact emitted by the contract.
    type Receipt;

    /// Stable contract name.
    const NAME: &'static str;

    /// Preconditions the exchange must satisfy before producing output.
    fn requires() -> &'static [&'static str];

    /// Postconditions the exchange promises for produced output.
    fn ensures() -> &'static [&'static str];

    /// Check the actual input, output, and receipt before the exchange succeeds.
    ///
    /// # Errors
    ///
    /// Rejects an exchange whose required transition assertions fail.
    fn verify_transition(
        input: &Self::Input,
        output: &Self::Output,
        receipt: &Self::Receipt,
    ) -> Result<(), BelvedereCertificationError>;

    /// Amenable standard IDs the exchange claims to satisfy.
    fn standard_ids() -> &'static [&'static str] {
        &[]
    }
}

#[tracing::instrument(level = "debug", skip(checks))]
pub(crate) fn require_transition<C: BelvedereContract>(
    checks: impl IntoIterator<Item = (&'static str, bool)>,
) -> Result<(), BelvedereCertificationError> {
    let checks = checks
        .into_iter()
        .map(|(name, passed)| {
            BelvedereInvariantCheck::new(
                name.to_owned(),
                passed,
                "input, output, and receipt do not satisfy this transition obligation".to_owned(),
            )
        })
        .collect::<Vec<_>>();
    BelvedereCertificationError::require_passing(C::NAME, &checks)
}
