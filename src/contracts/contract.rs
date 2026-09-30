//! Shared contract interface for Belvedere pipeline obligations.

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
}
