//! Certificates emitted from Belvedere pipeline receipts.

use std::{collections::BTreeMap, fmt::Display};

use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use super::{IngestionReceipt, LandscapeReceipt, UiIrReceipt, ViewModelReceipt};

/// Named stage in a complete Belvedere pipeline run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BelvederePipelineStage {
    /// Registry dump ingestion.
    RegistryIngestion,
    /// Proof assessment stream ingestion.
    AssessmentIngestion,
    /// Landscape graph construction.
    LandscapeModeling,
    /// Auditor navigation view-model construction.
    AuditorNavigation,
    /// `elicit_ui` IR projection.
    UiIrProjection,
}

impl BelvederePipelineStage {
    /// Stable stage identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryIngestion => "registry_ingestion",
            Self::AssessmentIngestion => "assessment_ingestion",
            Self::LandscapeModeling => "landscape_modeling",
            Self::AuditorNavigation => "auditor_navigation",
            Self::UiIrProjection => "ui_ir_projection",
        }
    }
}

impl Display for BelvederePipelineStage {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

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

/// Certificate for one successful Belvedere pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereStageCertificate {
    id: String,
    #[getter(skip)]
    stage: BelvederePipelineStage,
    contract: String,
    input_ids: Vec<String>,
    output_ids: Vec<String>,
    counts: BTreeMap<String, usize>,
    warning_count: usize,
    digest: String,
}

impl BelvedereStageCertificate {
    /// Certificate the registry dump ingestion stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn registry_ingestion(receipt: &IngestionReceipt) -> Self {
        Self::ingestion(BelvederePipelineStage::RegistryIngestion, receipt)
    }

    /// Certificate the assessment stream ingestion stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn assessment_ingestion(receipt: &IngestionReceipt) -> Self {
        Self::ingestion(BelvederePipelineStage::AssessmentIngestion, receipt)
    }

    /// Certificate the landscape modeling stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn landscape_modeling(receipt: &LandscapeReceipt) -> Self {
        let counts = BTreeMap::from([
            ("assessments".to_owned(), receipt.assessment_count()),
            ("claims".to_owned(), receipt.claim_count()),
            ("edges".to_owned(), receipt.edge_count()),
            ("gaps".to_owned(), receipt.gap_count()),
            ("proofs".to_owned(), receipt.proof_count()),
            ("roots".to_owned(), receipt.root_count()),
            ("warnings".to_owned(), receipt.warnings().len()),
        ]);
        Self::new(
            BelvederePipelineStage::LandscapeModeling,
            receipt.contract().to_owned(),
            receipt.source_record_ids().clone(),
            vec!["landscape_graph".to_owned()],
            counts,
            receipt.warnings().len(),
        )
    }

    /// Certificate the auditor navigation view-model stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn auditor_navigation(receipt: &ViewModelReceipt) -> Self {
        let counts = BTreeMap::from([
            (
                "assessment_items".to_owned(),
                receipt.counts().assessment_items(),
            ),
            ("claim_cards".to_owned(), receipt.counts().claim_cards()),
            ("gap_items".to_owned(), receipt.counts().gap_items()),
            ("root_items".to_owned(), receipt.counts().root_items()),
            ("warnings".to_owned(), receipt.warnings().len()),
            ("witness_items".to_owned(), receipt.counts().witness_items()),
        ]);
        Self::new(
            BelvederePipelineStage::AuditorNavigation,
            receipt.contract().to_owned(),
            receipt.source_record_ids().clone(),
            vec!["auditor_navigation_model".to_owned()],
            counts,
            receipt.warnings().len(),
        )
    }

    /// Certificate the `elicit_ui` projection stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn ui_ir_projection(receipt: &UiIrReceipt) -> Self {
        let counts = BTreeMap::from([
            (
                "minted_standards".to_owned(),
                receipt.minted_standards().len(),
            ),
            ("sections".to_owned(), receipt.counts().section_count()),
            ("trace_refs".to_owned(), receipt.counts().trace_ref_count()),
            ("widgets".to_owned(), receipt.counts().widget_count()),
            ("tree_nodes".to_owned(), receipt.counts().node_count()),
        ]);
        Self::new(
            BelvederePipelineStage::UiIrProjection,
            receipt.contract().to_owned(),
            receipt.source_record_ids().clone(),
            receipt.minted_standards().clone(),
            counts,
            0,
        )
    }

    /// Pipeline stage named by this certificate.
    #[must_use]
    pub const fn stage(&self) -> BelvederePipelineStage {
        self.stage
    }

    #[instrument(level = "debug", skip(stage, receipt))]
    fn ingestion(stage: BelvederePipelineStage, receipt: &IngestionReceipt) -> Self {
        let mut counts = receipt.decoded_counts().clone();
        counts.insert(
            "unsupported_fields".to_owned(),
            receipt.unsupported_fields().len(),
        );
        counts.insert("warnings".to_owned(), receipt.warnings().len());
        Self::new(
            stage,
            receipt.contract().to_owned(),
            vec![format!(
                "{}:{}:{}:{}",
                receipt.artifact_kind(),
                receipt.format(),
                receipt.path().display(),
                receipt.digest()
            )],
            receipt.source_record_ids().clone(),
            counts,
            receipt.warnings().len(),
        )
    }

    #[instrument(level = "debug", skip(stage, counts))]
    fn new(
        stage: BelvederePipelineStage,
        contract: String,
        input_ids: Vec<String>,
        output_ids: Vec<String>,
        counts: BTreeMap<String, usize>,
        warning_count: usize,
    ) -> Self {
        let digest = stage_digest(stage, &contract, &input_ids, &output_ids, &counts);
        let id = format!("belvedere.stage.{stage}:{digest}");
        Self {
            id,
            stage,
            contract,
            input_ids,
            output_ids,
            counts,
            warning_count,
            digest,
        }
    }
}

/// Certificate for a complete Belvedere pipeline run.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereRunCertificate {
    id: String,
    belvedere_version: String,
    invocation: BelvedereRunInvocation,
    input_artifact_ids: Vec<String>,
    stage_certificates: Vec<BelvedereStageCertificate>,
    stage_certificate_ids: Vec<String>,
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
        let belvedere_version = env!("CARGO_PKG_VERSION").to_owned();
        let digest = run_digest(
            &belvedere_version,
            &invocation,
            &input_artifact_ids,
            &stage_certificate_ids,
        );
        let id = format!("belvedere.run:{digest}");
        Self {
            id,
            belvedere_version,
            invocation,
            input_artifact_ids,
            stage_certificates,
            stage_certificate_ids,
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

#[instrument(level = "debug", skip(stage, counts))]
fn stage_digest(
    stage: BelvederePipelineStage,
    contract: &str,
    input_ids: &[String],
    output_ids: &[String],
    counts: &BTreeMap<String, usize>,
) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, stage.as_str());
    update_field(&mut hasher, contract);
    update_fields(&mut hasher, input_ids);
    update_fields(&mut hasher, output_ids);
    for (name, count) in counts {
        update_field(&mut hasher, name);
        update_field(&mut hasher, &count.to_string());
    }
    hex_digest(hasher)
}

#[instrument(level = "info", skip(invocation))]
fn run_digest(
    belvedere_version: &str,
    invocation: &BelvedereRunInvocation,
    input_artifact_ids: &[String],
    stage_certificate_ids: &[String],
) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, belvedere_version);
    update_field(&mut hasher, invocation.surface().as_str());
    update_fields(&mut hasher, invocation.arguments());
    update_fields(&mut hasher, input_artifact_ids);
    update_fields(&mut hasher, stage_certificate_ids);
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(hasher, fields))]
fn update_fields(hasher: &mut Sha256, fields: &[String]) {
    for field in fields {
        update_field(hasher, field);
    }
}

#[instrument(level = "trace", skip(hasher))]
fn update_field(hasher: &mut Sha256, field: &str) {
    hasher.update(field.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(field.as_bytes());
    hasher.update(b";");
}

#[instrument(level = "trace", skip(hasher))]
fn hex_digest(hasher: Sha256) -> String {
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
