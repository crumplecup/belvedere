//! End-to-end Belvedere pipeline runner.

use std::path::PathBuf;

use derive_builder::Builder;
use derive_getters::Getters;
use tracing::instrument;

use crate::{
    certify::{
        BelvedereRunCertificate, BelvedereRunInvocation, IngestionReceipt, LandscapeReceipt,
        UiIrReceipt, ViewModelReceipt,
    },
    collect::read_raw_artifact,
    contracts::{ArtifactKind, LandscapeInput, UiIrInput, UiIrProjection, ViewModelInput},
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::{
        AssessmentStreamIngestion, AuditorNavigationProjection, ElicitUiProjection,
        LandscapeModeling, RegistryDumpIngestion,
    },
};

/// Input artifact paths for a normal Belvedere pipeline run.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct BelvedereRunInput {
    registry_path: PathBuf,
    assessments_path: PathBuf,
    invocation: BelvedereRunInvocation,
}

impl BelvedereRunInput {
    /// Create run input for direct library use.
    #[instrument(level = "debug", skip(registry_path, assessments_path))]
    #[must_use]
    pub fn new(registry_path: impl Into<PathBuf>, assessments_path: impl Into<PathBuf>) -> Self {
        Self::with_invocation(
            registry_path,
            assessments_path,
            BelvedereRunInvocation::library(),
        )
    }

    /// Create run input with explicit invocation provenance.
    #[instrument(level = "trace", skip(registry_path, assessments_path, invocation))]
    #[must_use]
    pub fn with_invocation(
        registry_path: impl Into<PathBuf>,
        assessments_path: impl Into<PathBuf>,
        invocation: BelvedereRunInvocation,
    ) -> Self {
        Self {
            registry_path: registry_path.into(),
            assessments_path: assessments_path.into(),
            invocation,
        }
    }
}

/// Evidence emitted by a complete Belvedere pipeline run.
#[derive(Debug, Clone, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct BelvedereRunReport {
    registry_receipt: IngestionReceipt,
    assessment_receipt: IngestionReceipt,
    landscape_receipt: LandscapeReceipt,
    view_model_receipt: ViewModelReceipt,
    ui_ir_receipt: UiIrReceipt,
    certificate: BelvedereRunCertificate,
    projection: UiIrProjection,
}

/// Run the normal Belvedere pipeline over Amenable artifact files.
///
/// # Errors
///
/// Returns an error if any artifact cannot be read or any exchange rejects its
/// input evidence.
#[instrument(level = "info", skip(input))]
pub fn run_belvedere_pipeline(
    input: BelvedereRunInput,
) -> Result<BelvedereRunReport, BelvedereError> {
    let registry_artifact = read_raw_artifact(input.registry_path(), ArtifactKind::RegistryDump)?;
    let assessment_artifact =
        read_raw_artifact(input.assessments_path(), ArtifactKind::ProofAssessments)?;

    let (registry, registry_receipt) = RegistryDumpIngestion.ingest(registry_artifact)?;
    let (assessments, assessment_receipt) =
        AssessmentStreamIngestion.ingest(assessment_artifact)?;
    let (landscape, landscape_receipt) =
        LandscapeModeling.model(LandscapeInput::new(registry, assessments))?;
    let (view_model, view_model_receipt) =
        AuditorNavigationProjection.project(ViewModelInput::new(landscape))?;
    let (projection, ui_ir_receipt) = ElicitUiProjection.project(UiIrInput::new(view_model))?;
    let certificate = BelvedereRunCertificate::from_receipts_with_invocation(
        input.invocation().clone(),
        &registry_receipt,
        &assessment_receipt,
        &landscape_receipt,
        &view_model_receipt,
        &ui_ir_receipt,
    );

    BelvedereRunReportBuilder::default()
        .registry_receipt(registry_receipt)
        .assessment_receipt(assessment_receipt)
        .landscape_receipt(landscape_receipt)
        .view_model_receipt(view_model_receipt)
        .ui_ir_receipt(ui_ir_receipt)
        .certificate(certificate)
        .projection(projection)
        .build()
        .map_err(|error| BelvedereConstructionError::new("BelvedereRunReport", error).into())
}
