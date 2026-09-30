//! Deterministic digest helpers for certificates.

use sha2::{Digest as _, Sha256};
use tracing::instrument;

#[instrument(level = "trace", skip(hasher, fields))]
pub(crate) fn update_fields(hasher: &mut Sha256, fields: &[String]) {
    for field in fields {
        update_field(hasher, field);
    }
}

#[instrument(level = "trace", skip(hasher))]
pub(crate) fn update_field(hasher: &mut Sha256, field: &str) {
    hasher.update(field.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(field.as_bytes());
    hasher.update(b";");
}

#[instrument(level = "trace", skip(hasher))]
pub(crate) fn hex_digest(hasher: Sha256) -> String {
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
