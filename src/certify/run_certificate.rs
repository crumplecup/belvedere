//! Run certificate construction.

use std::collections::BTreeMap;

use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BelvedereInvariantCheck, BelvedereStageCertificate, IngestionReceipt, LandscapeReceipt,
    RunInputArtifactEvidence, RunInvariantEvidence, RunStageCertificateEvidence, UiIrReceipt,
    ViewModelReceipt, hex_digest, run_invariant_checks, update_field, update_fields,
};
use crate::contracts::{
    BelvederePipelineClaim, REPORT_REPRODUCIBILITY_STANDARD_ID,
    belvedere_pipeline_claims_for_standard_ids,
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
    component_versions: BTreeMap<String, String>,
    invocation: BelvedereRunInvocation,
    pipeline_claims: Vec<BelvederePipelineClaim>,
    pipeline_claim_phase_counts: BTreeMap<String, usize>,
    input_artifact_ids: Vec<String>,
    input_artifact_digests: Vec<String>,
    stage_certificates: Vec<BelvedereStageCertificate>,
    stage_certificate_ids: Vec<String>,
    stage_certificate_digests: Vec<String>,
    stage_kind_counts: BTreeMap<String, usize>,
    invariant_checks: Vec<BelvedereInvariantCheck>,
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
            artifact_id_with_role("registry", registry_receipt),
            artifact_id_with_role("assessment", assessment_receipt),
        ];
        let input_artifact_digests = vec![
            registry_receipt.digest().to_owned(),
            assessment_receipt.digest().to_owned(),
        ];
        Self::from_stage_certificates(
            invocation,
            input_artifact_ids,
            input_artifact_digests,
            stage_certificates,
        )
    }

    #[instrument(
        level = "debug",
        skip(
            invocation,
            input_artifact_ids,
            input_artifact_digests,
            stage_certificates
        )
    )]
    pub(crate) fn from_stage_certificates(
        invocation: BelvedereRunInvocation,
        input_artifact_ids: Vec<String>,
        input_artifact_digests: Vec<String>,
        stage_certificates: Vec<BelvedereStageCertificate>,
    ) -> Self {
        let stage_certificate_ids = stage_certificates
            .iter()
            .map(|certificate| certificate.id().to_owned())
            .collect::<Vec<_>>();
        let stage_certificate_digests = stage_certificates
            .iter()
            .map(|certificate| certificate.digest().to_owned())
            .collect::<Vec<_>>();
        let stage_kind_counts = stage_kind_counts(&stage_certificates);
        let stage_certificate_digest_matches = stage_certificates
            .iter()
            .zip(&stage_certificate_digests)
            .all(|(certificate, digest)| certificate.digest() == digest);
        let pipeline_claims = pipeline_claims_for_stage_certificates(&stage_certificates);
        let pipeline_claim_ids = pipeline_claims
            .iter()
            .map(|claim| claim.claim_id().to_owned())
            .collect::<Vec<_>>();
        let pipeline_claim_phase_counts = pipeline_claim_phase_counts(&pipeline_claims);
        let belvedere_version = env!("CARGO_PKG_VERSION").to_owned();
        let component_versions = component_versions(&belvedere_version);
        let invariant_checks = run_invariant_checks(RunInvariantEvidence::new(
            &component_versions,
            invocation.surface().as_str(),
            invocation.arguments(),
            &pipeline_claim_ids,
            &pipeline_claim_phase_counts,
            RunInputArtifactEvidence::new(&input_artifact_ids, &input_artifact_digests),
            RunStageCertificateEvidence::new(
                &stage_certificate_ids,
                &stage_certificate_digests,
                &stage_kind_counts,
                stage_certificate_digest_matches,
            ),
        ));
        let digest = run_digest(RunDigestEvidence {
            belvedere_version: &belvedere_version,
            component_versions: &component_versions,
            invocation: &invocation,
            pipeline_claims: &pipeline_claims,
            pipeline_claim_phase_counts: &pipeline_claim_phase_counts,
            input_artifact_ids: &input_artifact_ids,
            input_artifact_digests: &input_artifact_digests,
            stage_certificate_ids: &stage_certificate_ids,
            stage_certificate_digests: &stage_certificate_digests,
            stage_kind_counts: &stage_kind_counts,
            invariant_checks: &invariant_checks,
        });
        let id = format!("belvedere.run:{digest}");
        Self {
            schema_id: BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID.to_owned(),
            schema_version: BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
            id,
            belvedere_version,
            component_versions,
            invocation,
            pipeline_claims,
            pipeline_claim_phase_counts,
            input_artifact_ids,
            input_artifact_digests,
            stage_certificates,
            stage_certificate_ids,
            stage_certificate_digests,
            stage_kind_counts,
            invariant_checks,
            digest,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RunDigestEvidence<'evidence> {
    belvedere_version: &'evidence str,
    component_versions: &'evidence BTreeMap<String, String>,
    invocation: &'evidence BelvedereRunInvocation,
    pipeline_claims: &'evidence [BelvederePipelineClaim],
    pipeline_claim_phase_counts: &'evidence BTreeMap<String, usize>,
    input_artifact_ids: &'evidence [String],
    input_artifact_digests: &'evidence [String],
    stage_certificate_ids: &'evidence [String],
    stage_certificate_digests: &'evidence [String],
    stage_kind_counts: &'evidence BTreeMap<String, usize>,
    invariant_checks: &'evidence [BelvedereInvariantCheck],
}

#[instrument(level = "trace", skip(receipt))]
fn artifact_id_with_role(role: &str, receipt: &IngestionReceipt) -> String {
    format!(
        "{role}:{}:{}:{}",
        receipt.artifact_kind(),
        receipt.path().display(),
        receipt.digest()
    )
}

#[instrument(level = "info", skip(evidence))]
fn run_digest(evidence: RunDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID);
    update_field(
        &mut hasher,
        &BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION.to_string(),
    );
    update_field(&mut hasher, evidence.belvedere_version);
    for (component, version) in evidence.component_versions {
        update_field(&mut hasher, component);
        update_field(&mut hasher, version);
    }
    update_field(&mut hasher, evidence.invocation.surface().as_str());
    update_fields(&mut hasher, evidence.invocation.arguments());
    for claim in evidence.pipeline_claims {
        update_field(&mut hasher, claim.claim_id());
        update_field(&mut hasher, claim.phase());
        update_field(&mut hasher, claim.description());
        update_field(&mut hasher, claim.standard_type());
    }
    for (phase, count) in evidence.pipeline_claim_phase_counts {
        update_field(&mut hasher, phase);
        update_field(&mut hasher, &count.to_string());
    }
    update_fields(&mut hasher, evidence.input_artifact_ids);
    update_fields(&mut hasher, evidence.input_artifact_digests);
    update_fields(&mut hasher, evidence.stage_certificate_ids);
    update_fields(&mut hasher, evidence.stage_certificate_digests);
    for (stage, count) in evidence.stage_kind_counts {
        update_field(&mut hasher, stage);
        update_field(&mut hasher, &count.to_string());
    }
    for check in evidence.invariant_checks {
        update_field(&mut hasher, check.name());
        update_field(&mut hasher, &check.passed().to_string());
        update_field(&mut hasher, check.detail());
    }
    hex_digest(hasher)
}

#[instrument(level = "debug", skip(stage_certificates))]
fn pipeline_claims_for_stage_certificates(
    stage_certificates: &[BelvedereStageCertificate],
) -> Vec<BelvederePipelineClaim> {
    let mut standard_ids = stage_certificates
        .iter()
        .flat_map(|certificate| certificate.standards().iter().cloned())
        .collect::<Vec<_>>();
    standard_ids.push(REPORT_REPRODUCIBILITY_STANDARD_ID.to_owned());
    standard_ids.sort();
    standard_ids.dedup();
    belvedere_pipeline_claims_for_standard_ids(&standard_ids)
}

#[instrument(level = "debug", skip(pipeline_claims))]
fn pipeline_claim_phase_counts(
    pipeline_claims: &[BelvederePipelineClaim],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for claim in pipeline_claims {
        *counts
            .entry(format!("phase_{}", claim.phase()))
            .or_insert(0) += 1;
    }
    counts
}

#[instrument(level = "debug", skip(stage_certificates))]
fn stage_kind_counts(stage_certificates: &[BelvedereStageCertificate]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for certificate in stage_certificates {
        *counts
            .entry(format!("stage_{}", certificate.stage().as_str()))
            .or_insert(0) += 1;
    }
    counts
}

#[instrument(level = "debug", skip(belvedere_version))]
fn component_versions(belvedere_version: &str) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("amenable".to_owned(), locked_package_version("amenable")),
        ("belvedere".to_owned(), belvedere_version.to_owned()),
        ("elicit_ui".to_owned(), locked_package_version("elicit_ui")),
        (
            "elicitation".to_owned(),
            locked_package_version("elicitation"),
        ),
    ])
}

#[instrument(level = "trace", skip(package_name))]
fn locked_package_version(package_name: &str) -> String {
    let mut package_matches = false;
    for line in include_str!("../../Cargo.lock").lines() {
        if let Some(name) = lock_value(line, "name") {
            package_matches = name == package_name;
            continue;
        }
        if package_matches && let Some(version) = lock_value(line, "version") {
            return version.to_owned();
        }
    }
    "unknown".to_owned()
}

#[instrument(level = "trace", skip(line))]
fn lock_value<'line>(line: &'line str, key: &str) -> Option<&'line str> {
    let prefix = format!("{key} = \"");
    line.strip_prefix(&prefix)?.strip_suffix('"')
}
