//! Digest construction for stage certificates.

use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereInvariantCheck, hex_digest, update_field, update_fields,
};

use super::StageCertificateEvidence;

#[instrument(level = "debug", skip(evidence, invariant_checks))]
pub(super) fn stage_digest(
    evidence: &StageCertificateEvidence,
    invariant_checks: &[BelvedereInvariantCheck],
) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID);
    update_field(
        &mut hasher,
        &BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION.to_string(),
    );
    update_field(&mut hasher, evidence.stage.as_str());
    update_field(&mut hasher, &evidence.contract);
    update_fields(&mut hasher, &evidence.standards);
    update_fields(&mut hasher, &evidence.requires);
    update_fields(&mut hasher, &evidence.ensures);
    update_fields(&mut hasher, &evidence.input_ids);
    update_fields(&mut hasher, &evidence.input_digests);
    update_fields(&mut hasher, &evidence.output_ids);
    for (name, count) in &evidence.counts {
        update_field(&mut hasher, name);
        update_field(&mut hasher, &count.to_string());
    }
    for check in invariant_checks {
        update_field(&mut hasher, check.name());
        update_field(&mut hasher, &check.passed().to_string());
        update_field(&mut hasher, check.kind().as_str());
        update_field(&mut hasher, check.detail());
    }
    hex_digest(hasher)
}
