//! Shared exchange interface for proof-bearing Belvedere transformations.

use crate::{
    certify::{BelvedereCertificationError, BelvedereTransitionReceipt},
    contracts::BelvedereContract,
};

/// A concrete transformation that implements a named contract.
pub trait BelvedereExchange<C>
where
    C: BelvedereContract,
{
    /// Error surface for failed exchanges.
    type Error: From<BelvedereCertificationError>;

    /// Build output evidence and a receipt from a borrowed input.
    fn transform(&self, input: &C::Input) -> Result<(C::Output, C::Receipt), Self::Error>;

    /// Perform the checked exchange, returning output evidence and its receipt.
    ///
    /// # Errors
    ///
    /// Rejects failed transformations and required transition assertions.
    fn exchange(&self, input: C::Input) -> Result<(C::Output, C::Receipt), Self::Error> {
        let (output, mut receipt) = self.transform(&input)?;
        let checks = C::verify_transition(&input, &output, &receipt)?;
        receipt.record_transition_checks(checks)?;
        C::verify_receipt(&receipt)?;
        Ok((output, receipt))
    }
}
