//! Shared contract interface for Belvedere pipeline obligations.

use crate::certify::{
    BelvedereCertificationError, BelvedereInvariantCheck, BelvedereTransitionReceipt,
};

/// A named Belvedere pipeline obligation.
pub trait BelvedereContract {
    /// Evidence accepted by the contract.
    type Input;
    /// Evidence produced by the contract.
    type Output;
    /// Audit artifact emitted by the contract.
    type Receipt: BelvedereTransitionReceipt;

    /// Stable contract name.
    const NAME: &'static str;

    /// Complete ordered set of required transition assertion names.
    const ASSERTION_NAMES: &'static [&'static str];

    /// Assertion names that are required input preconditions.
    const PRECONDITION_NAMES: &'static [&'static str] = &[];

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
    ) -> Result<Vec<BelvedereInvariantCheck>, BelvedereCertificationError>;

    /// Check the attested receipt after transition assertions are recorded.
    ///
    /// # Errors
    ///
    /// Rejects a receipt that cannot yield a successful stage certificate.
    fn verify_receipt(receipt: &Self::Receipt) -> Result<(), BelvedereCertificationError>;

    /// Amenable standard IDs the exchange claims to satisfy.
    fn standard_ids() -> &'static [&'static str] {
        &[]
    }
}

#[tracing::instrument(level = "debug", skip(checks))]
pub(crate) fn require_transition<C: BelvedereContract>(
    checks: impl IntoIterator<Item = (&'static str, bool)>,
) -> Result<Vec<BelvedereInvariantCheck>, BelvedereCertificationError> {
    let checks = checks
        .into_iter()
        .map(|(name, passed)| {
            let detail =
                "input, output, and receipt do not satisfy this transition obligation".to_owned();
            if C::PRECONDITION_NAMES.contains(&name) {
                BelvedereInvariantCheck::precondition(name.to_owned(), passed, detail)
            } else {
                BelvedereInvariantCheck::new(name.to_owned(), passed, detail)
            }
        })
        .collect::<Vec<_>>();
    let names = checks
        .iter()
        .map(|check| check.name().as_str())
        .collect::<Vec<_>>();
    let complete = names == C::ASSERTION_NAMES;
    BelvedereCertificationError::require_passing(
        C::NAME,
        &[BelvedereInvariantCheck::new(
            "transition_assertion_set_complete".to_owned(),
            complete,
            format!("expected {:?}; observed {names:?}", C::ASSERTION_NAMES),
        )],
    )?;
    BelvedereCertificationError::require_passing(C::NAME, &checks)?;
    Ok(checks)
}
