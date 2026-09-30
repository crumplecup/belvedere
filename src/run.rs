//! End-to-end Belvedere pipeline runner.

use std::path::PathBuf;

use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use tracing::instrument;

use crate::{
    certify::{
        BelvedereRunCertificate, IngestionReceipt, LandscapeReceipt, UiIrReceipt, ViewModelReceipt,
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
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct BelvedereRunInput {
    #[new(into)]
    registry_path: PathBuf,
    #[new(into)]
    assessments_path: PathBuf,
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
    let certificate = BelvedereRunCertificate::from_receipts(
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
