//! Witness surface projections.

use tracing::instrument;

use crate::{
    contracts::{LandscapeClaim, WitnessSurfaceItem, WitnessSurfaceItemBuilder},
    error::BelvedereConstructionError,
};

use super::trace;

#[instrument(level = "debug", skip(claims, source_record_ids))]
pub(super) fn project_witness_surface<'a>(
    claims: impl Iterator<Item = &'a LandscapeClaim>,
    source_record_ids: &[String],
) -> Result<Vec<WitnessSurfaceItem>, BelvedereConstructionError> {
    let mut items = Vec::new();
    for claim in claims {
        for witness in claim.witness_surfaces() {
            let item = WitnessSurfaceItemBuilder::default()
                .claim_id(claim.id())
                .verifier(witness.verifier())
                .destination_module(witness.destination_module())
                .support_kind(witness.support_kind())
                .trivial(witness.trivial())
                .checked(witness.checked())
                .trusted(witness.trusted())
                .opaque(witness.opaque())
                .trace(trace(claim.id(), source_record_ids))
                .build()
                .map_err(|error| BelvedereConstructionError::new("WitnessSurfaceItem", error))?;
            items.push(item);
        }
    }
    Ok(items)
}
