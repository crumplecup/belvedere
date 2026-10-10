//! Proof token counts for UI IR receipts.

use std::collections::BTreeMap;

use tracing::instrument;

use crate::contracts::UiViewportVerification;

#[instrument(level = "debug", skip(viewport_verifications))]
pub(super) fn proof_token_counts(
    viewport_verifications: &[UiViewportVerification],
) -> BTreeMap<String, usize> {
    BTreeMap::from([
        ("ui_ir_proof_token_ir_sourced".to_owned(), 1),
        ("ui_ir_proof_token_wcag_verified".to_owned(), 1),
        (
            "ui_ir_proof_token_viewport_wcag_verified".to_owned(),
            viewport_verifications.len(),
        ),
    ])
}
