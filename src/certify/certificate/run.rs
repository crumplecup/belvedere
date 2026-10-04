//! Run certificate construction.

use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{IngestionReceipt, LandscapeReceipt, UiIrReceipt, ViewModelReceipt};

use super::{
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BelvedereStageCertificate, hex_digest, update_field, update_fields,
};

/// Public invocation surface that produced a Belvedere run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BelvedereInvocationSurface {
    /// Belvedere was called through the library API.
    Library,
    /// Belvedere was called through the command-line API.
    Cli,
}

impl BelvedereInvocationSurface {
    /// Stable invocation surface identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Cli => "cli",
        }
    }
}

/// Invocation provenance recorded in a run certificate.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereRunInvocation {
    #[getter(skip)]
    surface: BelvedereInvocationSurface,
    arguments: Vec<String>,
}

impl BelvedereRunInvocation {
    /// Invocation provenance for direct library calls.
    #[must_use]
    pub const fn library() -> Self {
        Self {
            surface: BelvedereInvocationSurface::Library,
            arguments: Vec::new(),
        }
    }

    /// Invocation provenance for command-line calls.
    #[instrument(level = "debug", skip(args))]
    #[must_use]
    pub fn cli(args: impl IntoIterator<Item = String>) -> Self {
        Self {
            surface: BelvedereInvocationSurface::Cli,
            arguments: args.into_iter().collect(),
        }
    }

    /// Public surface that produced the run.
    #[must_use]
    pub const fn surface(&self) -> BelvedereInvocationSurface {
        self.surface
    }
}

/// Certificate for a complete Belvedere pipeline run.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereRunCertificate {
    schema_id: String,
    schema_version: u16,
    id: String,
    belvedere_version: String,
    invocation: BelvedereRunInvocation,
    input_artifact_ids: Vec<String>,
    stage_certificates: Vec<BelvedereStageCertificate>,
    stage_certificate_ids: Vec<String>,
    stage_certificate_digests: Vec<String>,
    digest: String,
}

impl BelvedereRunCertificate {
    /// Build a run certificate from the receipts emitted by each stage.
    #[instrument(
        level = "debug",
        skip(
            registry_receipt,
            assessment_receipt,
            landscape_receipt,
            view_model_receipt,
            ui_ir_receipt
        )
    )]
    #[must_use]
    pub fn from_receipts(
        registry_receipt: &IngestionReceipt,
        assessment_receipt: &IngestionReceipt,
        landscape_receipt: &LandscapeReceipt,
        view_model_receipt: &ViewModelReceipt,
        ui_ir_receipt: &UiIrReceipt,
    ) -> Self {
        Self::from_receipts_with_invocation(
            BelvedereRunInvocation::library(),
            registry_receipt,
            assessment_receipt,
            landscape_receipt,
            view_model_receipt,
            ui_ir_receipt,
        )
    }

    /// Build a run certificate from a recorded invocation and stage receipts.
    #[instrument(
        level = "debug",
        skip(
            invocation,
            registry_receipt,
            assessment_receipt,
            landscape_receipt,
            view_model_receipt,
            ui_ir_receipt
        )
    )]
    #[must_use]
    pub fn from_receipts_with_invocation(
        invocation: BelvedereRunInvocation,
        registry_receipt: &IngestionReceipt,
        assessment_receipt: &IngestionReceipt,
        landscape_receipt: &LandscapeReceipt,
        view_model_receipt: &ViewModelReceipt,
        ui_ir_receipt: &UiIrReceipt,
    ) -> Self {
        let stage_certificates = vec![
            BelvedereStageCertificate::registry_ingestion(registry_receipt),
            BelvedereStageCertificate::assessment_ingestion(assessment_receipt),
            BelvedereStageCertificate::landscape_modeling(landscape_receipt),
            BelvedereStageCertificate::auditor_navigation(view_model_receipt),
            BelvedereStageCertificate::ui_ir_projection(ui_ir_receipt),
        ];
        let input_artifact_ids = vec![
            artifact_id(registry_receipt),
            artifact_id(assessment_receipt),
        ];
        let stage_certificate_ids = stage_certificates
            .iter()
            .map(|certificate| certificate.id().to_owned())
            .collect::<Vec<_>>();
        let stage_certificate_digests = stage_certificates
            .iter()
            .map(|certificate| certificate.digest().to_owned())
            .collect::<Vec<_>>();
        let belvedere_version = env!("CARGO_PKG_VERSION").to_owned();
        let digest = run_digest(
            &belvedere_version,
            &invocation,
            &input_artifact_ids,
            &stage_certificate_ids,
            &stage_certificate_digests,
        );
        let id = format!("belvedere.run:{digest}");
        Self {
            schema_id: BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID.to_owned(),
            schema_version: BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
            id,
            belvedere_version,
            invocation,
            input_artifact_ids,
            stage_certificates,
            stage_certificate_ids,
            stage_certificate_digests,
            digest,
        }
    }
}

#[instrument(level = "trace", skip(receipt))]
fn artifact_id(receipt: &IngestionReceipt) -> String {
    format!(
        "{}:{}:{}",
        receipt.artifact_kind(),
        receipt.path().display(),
        receipt.digest()
    )
}

#[instrument(level = "info", skip(invocation))]
fn run_digest(
    belvedere_version: &str,
    invocation: &BelvedereRunInvocation,
    input_artifact_ids: &[String],
    stage_certificate_ids: &[String],
    stage_certificate_digests: &[String],
) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID);
    update_field(
        &mut hasher,
        &BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION.to_string(),
    );
    update_field(&mut hasher, belvedere_version);
    update_field(&mut hasher, invocation.surface().as_str());
    update_fields(&mut hasher, invocation.arguments());
    update_fields(&mut hasher, input_artifact_ids);
    update_fields(&mut hasher, stage_certificate_ids);
    update_fields(&mut hasher, stage_certificate_digests);
    hex_digest(hasher)
}
