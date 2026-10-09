//! Helpers for copying contract metadata into certificates.

use tracing::instrument;

use crate::contracts::BelvedereContract;

#[instrument(level = "trace")]
pub(super) fn contract_requirements<C: BelvedereContract>() -> Vec<String> {
    C::requires()
        .iter()
        .map(|requirement| (*requirement).to_owned())
        .collect()
}

#[instrument(level = "trace")]
pub(super) fn contract_standards<C: BelvedereContract>() -> Vec<String> {
    C::standard_ids()
        .iter()
        .map(|standard| (*standard).to_owned())
        .collect()
}

#[instrument(level = "trace")]
pub(super) fn contract_guarantees<C: BelvedereContract>() -> Vec<String> {
    C::ensures()
        .iter()
        .map(|guarantee| (*guarantee).to_owned())
        .collect()
}
