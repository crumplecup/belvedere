//! End-to-end Belvedere pipeline runner.

use std::path::{Path, PathBuf};

use derive_builder::Builder;
use derive_getters::Getters;
use derive_new::new;
use tracing::instrument;

use crate::{
    certify::{
        BelvedereArtifactManifest, BelvedereDiffManifestReceipts, BelvedereRunCertificate,
        BelvedereRunInvocation, BelvedereSnapshotManifestReceipts, BelvedereStageCertificate,
        IngestionReceipt, LandscapeDiffReceipt, LandscapeReceipt, UiIrReceipt, ViewModelReceipt,
    },
    collect::read_raw_artifact,
    contracts::{
        ArtifactKind, AuditorNavigationModel, LandscapeDiff, LandscapeDiffInput, LandscapeGraph,
        LandscapeInput, UiIrInput, UiIrProjection, ViewModelInput,
    },
    error::{BelvedereConstructionError, BelvedereError},
    exchanges::{
        AssessmentStreamIngestion, AuditorNavigationProjection, ElicitUiProjection,
        LandscapeDiffing, LandscapeModeling, RegistryDumpIngestion,
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

/// Input for comparing two Belvedere landscape snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct BelvedereLandscapeDiffRunInput {
    previous: BelvedereRunInput,
    current: BelvedereRunInput,
}

/// Evidence emitted by a complete Belvedere pipeline run.
#[derive(Debug, Clone, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct BelvedereRunReport {
    registry_receipt: IngestionReceipt,
    assessment_receipt: IngestionReceipt,
    landscape: LandscapeGraph,
    landscape_receipt: LandscapeReceipt,
    view_model: AuditorNavigationModel,
    view_model_receipt: ViewModelReceipt,
    ui_ir_receipt: UiIrReceipt,
    certificate: BelvedereRunCertificate,
    manifest: BelvedereArtifactManifest,
    projection: UiIrProjection,
}

/// Evidence emitted by a landscape diff run over two artifact sets.
#[derive(Debug, Clone, Getters, Builder)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct BelvedereLandscapeDiffRunReport {
    previous_registry_receipt: IngestionReceipt,
    previous_assessment_receipt: IngestionReceipt,
    previous_landscape: LandscapeGraph,
    previous_landscape_receipt: LandscapeReceipt,
    previous_view_model: AuditorNavigationModel,
    previous_view_model_receipt: ViewModelReceipt,
    previous_ui_ir_receipt: UiIrReceipt,
    current_registry_receipt: IngestionReceipt,
    current_assessment_receipt: IngestionReceipt,
    current_landscape: LandscapeGraph,
    current_landscape_receipt: LandscapeReceipt,
    current_view_model: AuditorNavigationModel,
    current_view_model_receipt: ViewModelReceipt,
    current_ui_ir_receipt: UiIrReceipt,
    diff_receipt: LandscapeDiffReceipt,
    certificate: BelvedereRunCertificate,
    manifest: BelvedereArtifactManifest,
    diff_certificate: BelvedereStageCertificate,
    previous_projection: UiIrProjection,
    current_projection: UiIrProjection,
    diff: LandscapeDiff,
}

#[derive(Debug)]
struct LandscapeRunSnapshot {
    landscape: LandscapeGraph,
    registry_receipt: IngestionReceipt,
    assessment_receipt: IngestionReceipt,
    landscape_receipt: LandscapeReceipt,
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
        LandscapeModeling.model(LandscapeInput::from_ingestion(
            registry,
            &registry_receipt,
            assessments,
            &assessment_receipt,
        ))?;
    let (view_model, view_model_receipt) =
        AuditorNavigationProjection.project(ViewModelInput::new(landscape.clone()))?;
    let (projection, ui_ir_receipt) =
        ElicitUiProjection.project(UiIrInput::new(view_model.clone()))?;
    let certificate = BelvedereRunCertificate::from_receipts_with_invocation(
        input.invocation().clone(),
        &registry_receipt,
        &assessment_receipt,
        &landscape_receipt,
        &view_model_receipt,
        &ui_ir_receipt,
    );
    let manifest = BelvedereArtifactManifest::normal_run(
        &certificate,
        &registry_receipt,
        &assessment_receipt,
        &landscape_receipt,
        &view_model_receipt,
        &ui_ir_receipt,
    );

    BelvedereRunReportBuilder::default()
        .registry_receipt(registry_receipt)
        .assessment_receipt(assessment_receipt)
        .landscape(landscape)
        .landscape_receipt(landscape_receipt)
        .view_model(view_model)
        .view_model_receipt(view_model_receipt)
        .ui_ir_receipt(ui_ir_receipt)
        .certificate(certificate)
        .manifest(manifest)
        .projection(projection)
        .build()
        .map_err(|error| BelvedereConstructionError::new("BelvedereRunReport", error).into())
}

/// Run a certified landscape diff over two ordinary Amenable artifact sets.
///
/// # Errors
///
/// Returns an error if artifact collection, ingestion, landscape modeling, the
/// diff exchange, or report construction fails.
#[instrument(level = "info", skip(input))]
pub fn run_belvedere_landscape_diff(
    input: BelvedereLandscapeDiffRunInput,
) -> Result<BelvedereLandscapeDiffRunReport, BelvedereError> {
    let invocation = input.current().invocation().clone();
    let previous = build_landscape_snapshot(
        input.previous().registry_path(),
        input.previous().assessments_path(),
    )?;
    let current = build_landscape_snapshot(
        input.current().registry_path(),
        input.current().assessments_path(),
    )?;
    let input_artifact_ids = diff_input_artifact_ids(&previous, &current);
    let input_artifact_digests = diff_input_artifact_digests(&previous, &current);
    let (previous_view_model, previous_view_model_receipt) =
        AuditorNavigationProjection.project(ViewModelInput::new(previous.landscape.clone()))?;
    let (previous_projection, previous_ui_ir_receipt) =
        ElicitUiProjection.project(UiIrInput::new(previous_view_model.clone()))?;
    let previous_stage_certificates = auditor_stage_certificates(
        &previous,
        &previous_view_model_receipt,
        &previous_ui_ir_receipt,
    );
    let (current_view_model, current_view_model_receipt) =
        AuditorNavigationProjection.project(ViewModelInput::new(current.landscape.clone()))?;
    let (current_projection, current_ui_ir_receipt) =
        ElicitUiProjection.project(UiIrInput::new(current_view_model.clone()))?;
    let current_stage_certificates = auditor_stage_certificates(
        &current,
        &current_view_model_receipt,
        &current_ui_ir_receipt,
    );

    let previous_landscape = previous.landscape.clone();
    let current_landscape = current.landscape.clone();
    let (diff, diff_receipt) = LandscapeDiffing.diff(LandscapeDiffInput::new(
        previous.landscape,
        current.landscape,
    ))?;
    let diff_certificate = BelvedereStageCertificate::landscape_diff(&diff_receipt);
    let certificate = BelvedereRunCertificate::from_stage_certificates(
        invocation,
        input_artifact_ids,
        input_artifact_digests,
        previous_stage_certificates
            .into_iter()
            .chain(current_stage_certificates)
            .chain([diff_certificate.clone()])
            .collect(),
    );
    let manifest = BelvedereArtifactManifest::landscape_diff_run(
        &certificate,
        BelvedereDiffManifestReceipts::new(
            BelvedereSnapshotManifestReceipts::new(
                &previous.registry_receipt,
                &previous.assessment_receipt,
                &previous.landscape_receipt,
                &previous_view_model_receipt,
                &previous_ui_ir_receipt,
            ),
            BelvedereSnapshotManifestReceipts::new(
                &current.registry_receipt,
                &current.assessment_receipt,
                &current.landscape_receipt,
                &current_view_model_receipt,
                &current_ui_ir_receipt,
            ),
            &diff_receipt,
        ),
    );

    BelvedereLandscapeDiffRunReportBuilder::default()
        .previous_registry_receipt(previous.registry_receipt)
        .previous_assessment_receipt(previous.assessment_receipt)
        .previous_landscape(previous_landscape)
        .previous_landscape_receipt(previous.landscape_receipt)
        .previous_view_model(previous_view_model)
        .previous_view_model_receipt(previous_view_model_receipt)
        .previous_ui_ir_receipt(previous_ui_ir_receipt)
        .current_registry_receipt(current.registry_receipt)
        .current_assessment_receipt(current.assessment_receipt)
        .current_landscape(current_landscape)
        .current_landscape_receipt(current.landscape_receipt)
        .current_view_model(current_view_model)
        .current_view_model_receipt(current_view_model_receipt)
        .current_ui_ir_receipt(current_ui_ir_receipt)
        .diff_receipt(diff_receipt)
        .certificate(certificate)
        .manifest(manifest)
        .diff_certificate(diff_certificate)
        .previous_projection(previous_projection)
        .current_projection(current_projection)
        .diff(diff)
        .build()
        .map_err(|error| {
            BelvedereConstructionError::new("BelvedereLandscapeDiffRunReport", error).into()
        })
}

#[instrument(level = "debug", skip(registry_path, assessments_path))]
fn build_landscape_snapshot(
    registry_path: &Path,
    assessments_path: &Path,
) -> Result<LandscapeRunSnapshot, BelvedereError> {
    let registry_artifact = read_raw_artifact(registry_path, ArtifactKind::RegistryDump)?;
    let assessment_artifact = read_raw_artifact(assessments_path, ArtifactKind::ProofAssessments)?;

    let (registry, registry_receipt) = RegistryDumpIngestion.ingest(registry_artifact)?;
    let (assessments, assessment_receipt) =
        AssessmentStreamIngestion.ingest(assessment_artifact)?;
    let (landscape, landscape_receipt) =
        LandscapeModeling.model(LandscapeInput::from_ingestion(
            registry,
            &registry_receipt,
            assessments,
            &assessment_receipt,
        ))?;

    Ok(LandscapeRunSnapshot {
        landscape,
        registry_receipt,
        assessment_receipt,
        landscape_receipt,
    })
}

#[instrument(level = "debug", skip(snapshot))]
fn landscape_snapshot_stage_certificates(
    snapshot: &LandscapeRunSnapshot,
) -> Vec<BelvedereStageCertificate> {
    vec![
        BelvedereStageCertificate::registry_ingestion(&snapshot.registry_receipt),
        BelvedereStageCertificate::assessment_ingestion(&snapshot.assessment_receipt),
        BelvedereStageCertificate::landscape_modeling(&snapshot.landscape_receipt),
    ]
}

#[instrument(level = "debug", skip(snapshot, view_model_receipt, ui_ir_receipt))]
fn auditor_stage_certificates(
    snapshot: &LandscapeRunSnapshot,
    view_model_receipt: &ViewModelReceipt,
    ui_ir_receipt: &UiIrReceipt,
) -> Vec<BelvedereStageCertificate> {
    landscape_snapshot_stage_certificates(snapshot)
        .into_iter()
        .chain([
            BelvedereStageCertificate::auditor_navigation(view_model_receipt),
            BelvedereStageCertificate::ui_ir_projection(ui_ir_receipt),
        ])
        .collect()
}

#[instrument(level = "debug", skip(previous, current))]
fn diff_input_artifact_ids(
    previous: &LandscapeRunSnapshot,
    current: &LandscapeRunSnapshot,
) -> Vec<String> {
    vec![
        artifact_id_with_role("previous_registry", &previous.registry_receipt),
        artifact_id_with_role("previous_assessment", &previous.assessment_receipt),
        artifact_id_with_role("current_registry", &current.registry_receipt),
        artifact_id_with_role("current_assessment", &current.assessment_receipt),
    ]
}

#[instrument(level = "debug", skip(previous, current))]
fn diff_input_artifact_digests(
    previous: &LandscapeRunSnapshot,
    current: &LandscapeRunSnapshot,
) -> Vec<String> {
    vec![
        previous.registry_receipt.digest().to_owned(),
        previous.assessment_receipt.digest().to_owned(),
        current.registry_receipt.digest().to_owned(),
        current.assessment_receipt.digest().to_owned(),
    ]
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
