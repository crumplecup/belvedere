//! Required exchange assertions carried from receipts into certificates.

use sha2::{Digest as _, Sha256};

use crate::certify::{
    BelvedereCertificationError, BelvedereInvariantCheck, hex_digest, update_field,
};

/// A receipt that records the assertions checked against its actual transition.
pub trait BelvedereTransitionReceipt {
    /// Required assertions recorded by the exchange.
    fn transition_checks(&self) -> &[BelvedereInvariantCheck];

    /// Bind passing assertions to this receipt's identity.
    ///
    /// # Errors
    ///
    /// Rejects empty, failed, or previously recorded assertions.
    fn record_transition_checks(
        &mut self,
        checks: Vec<BelvedereInvariantCheck>,
    ) -> Result<(), BelvedereCertificationError>;
}

#[tracing::instrument(level = "trace", skip(stored, digest, id, checks))]
pub(crate) fn bind_transition_checks(
    stored: &mut Vec<BelvedereInvariantCheck>,
    digest: &mut String,
    id: &mut String,
    checks: Vec<BelvedereInvariantCheck>,
) -> Result<(), BelvedereCertificationError> {
    require_transition_checks(&checks)?;
    if !stored.is_empty() {
        return BelvedereCertificationError::require_passing(
            "transition",
            &[BelvedereInvariantCheck::new(
                "transition_assertions_recorded_once".to_owned(),
                false,
                "transition assertions already recorded".to_owned(),
            )],
        );
    }
    let (attested_id, attested_digest) = transition_identity(id, digest, &checks);
    *digest = attested_digest;
    *id = attested_id;
    *stored = checks;
    Ok(())
}

#[tracing::instrument(level = "trace", skip(checks))]
pub(crate) fn transition_identity(
    base_id: &str,
    base_digest: &str,
    checks: &[BelvedereInvariantCheck],
) -> (String, String) {
    let prefix = base_id
        .rsplit_once(':')
        .map_or(base_id, |(prefix, _)| prefix);
    let mut hasher = Sha256::new();
    update_field(&mut hasher, "belvedere.receipt.transition.v1");
    update_field(&mut hasher, base_digest);
    for check in checks {
        update_field(&mut hasher, check.name());
        update_field(&mut hasher, if check.passed() { "true" } else { "false" });
        update_field(&mut hasher, check.kind().as_str());
        update_field(&mut hasher, check.detail());
    }
    let digest = hex_digest(hasher);
    (format!("{prefix}:{digest}"), digest)
}

#[tracing::instrument(level = "trace", skip(checks))]
pub(crate) fn require_transition_checks(
    checks: &[BelvedereInvariantCheck],
) -> Result<(), BelvedereCertificationError> {
    if checks.is_empty() {
        return BelvedereCertificationError::require_passing(
            "transition",
            &[BelvedereInvariantCheck::new(
                "transition_assertions_present".to_owned(),
                false,
                "required transition assertions were not recorded".to_owned(),
            )],
        );
    }
    BelvedereCertificationError::require_passing("transition", checks)
}

macro_rules! impl_attested_receipt {
    ($receipt:ty) => {
        impl crate::certify::BelvedereTransitionReceipt for $receipt {
            fn transition_checks(&self) -> &[crate::certify::BelvedereInvariantCheck] {
                &self.transition_checks
            }

            fn record_transition_checks(
                &mut self,
                checks: Vec<crate::certify::BelvedereInvariantCheck>,
            ) -> Result<(), crate::certify::BelvedereCertificationError> {
                crate::certify::transition::bind_transition_checks(
                    &mut self.transition_checks,
                    &mut self.receipt_digest,
                    &mut self.receipt_id,
                    checks,
                )
            }
        }
    };
}

pub(crate) use impl_attested_receipt;
