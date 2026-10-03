//! Backend coverage projections.

use tracing::instrument;

use crate::{
    contracts::{BackendCoverageItem, BackendCoverageItemBuilder, LandscapeClaim},
    error::BelvedereConstructionError,
};

use super::trace;

pub(super) const TARGET_VERIFIERS: [&str; 3] = ["kani", "creusot", "verus"];

#[instrument(level = "debug", skip(claims, source_record_ids))]
pub(super) fn project_backend_coverage<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    source_record_ids: &[String],
) -> Result<Vec<BackendCoverageItem>, BelvedereConstructionError> {
    let mut items = Vec::new();
    for claim in claims {
        for verifier in TARGET_VERIFIERS {
            items.push(
                BackendCoverageItemBuilder::default()
                    .claim_id(claim.id())
                    .verifier(verifier)
                    .covered(claim.proofs().contains(verifier))
                    .trace(trace(claim.id(), source_record_ids))
                    .build()
                    .map_err(|error| {
                        BelvedereConstructionError::new("BackendCoverageItem", error)
                    })?,
            );
        }
    }
    Ok(items)
}
