//! Shared exchange interface for proof-bearing Belvedere transformations.

use crate::contracts::BelvedereContract;

/// A concrete transformation that implements a named contract.
pub trait BelvedereExchange<C>
where
    C: BelvedereContract,
{
    /// Error surface for failed exchanges.
    type Error;

    /// Perform the exchange, returning output evidence and its receipt.
    fn exchange(&self, input: C::Input) -> Result<(C::Output, C::Receipt), Self::Error>;
}
