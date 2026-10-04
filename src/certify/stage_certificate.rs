//! Stage certificate construction.

use std::{collections::BTreeMap, fmt::Display};

use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::{
    certify::{
        BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
        BelvedereInvariantCheck, IngestionReceipt, LandscapeDiffReceipt, LandscapeReceipt,
        UiIrReceipt, ViewModelReceipt, hex_digest, stage_invariant_checks, update_field,
        update_fields,
    },
    contracts::{
        AssessmentStreamIngestionContract, AuditorNavigationContract, BelvedereContract,
        ElicitUiProjectionContract, LandscapeDiffContract, LandscapeModelingContract,
        RegistryDumpIngestionContract,
    },
};

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
    /// Landscape snapshot comparison.
    LandscapeDiff,
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
            Self::LandscapeDiff => "landscape_diff",
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

/// Certificate for one successful Belvedere pipeline stage.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereStageCertificate {
    schema_id: String,
    schema_version: u16,
    id: String,
    #[getter(skip)]
    stage: BelvederePipelineStage,
    contract: String,
    standards: Vec<String>,
    requires: Vec<String>,
    ensures: Vec<String>,
    input_ids: Vec<String>,
    output_ids: Vec<String>,
    counts: BTreeMap<String, usize>,
    invariant_checks: Vec<BelvedereInvariantCheck>,
    warning_count: usize,
    digest: String,
}

#[derive(Debug)]
struct StageCertificateEvidence {
    stage: BelvederePipelineStage,
    contract: String,
    standards: Vec<String>,
    requires: Vec<String>,
    ensures: Vec<String>,
    input_ids: Vec<String>,
    output_ids: Vec<String>,
    counts: BTreeMap<String, usize>,
    warning_count: usize,
}

impl BelvedereStageCertificate {
    /// Certificate the registry dump ingestion stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn registry_ingestion(receipt: &IngestionReceipt) -> Self {
        Self::ingestion::<RegistryDumpIngestionContract>(
            BelvederePipelineStage::RegistryIngestion,
            receipt,
        )
    }

    /// Certificate the assessment stream ingestion stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn assessment_ingestion(receipt: &IngestionReceipt) -> Self {
        Self::ingestion::<AssessmentStreamIngestionContract>(
            BelvederePipelineStage::AssessmentIngestion,
            receipt,
        )
    }

    /// Certificate the landscape modeling stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn landscape_modeling(receipt: &LandscapeReceipt) -> Self {
        let mut counts = BTreeMap::from([
            ("assessments".to_owned(), receipt.assessment_count()),
            ("claims".to_owned(), receipt.claim_count()),
            ("edges".to_owned(), receipt.edge_count()),
            ("gaps".to_owned(), receipt.gap_count()),
            (
                "landscape_items".to_owned(),
                receipt.landscape_item_ids().len(),
            ),
            ("proofs".to_owned(), receipt.proof_count()),
            ("roots".to_owned(), receipt.root_count()),
            ("warnings".to_owned(), receipt.warnings().len()),
        ]);
        counts.extend(receipt.gap_kind_counts().clone());
        counts.extend(receipt.assessment_recency_counts().clone());
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::LandscapeModeling,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<LandscapeModelingContract>(),
            requires: contract_requirements::<LandscapeModelingContract>(),
            ensures: contract_guarantees::<LandscapeModelingContract>(),
            input_ids: receipt.source_record_ids().clone(),
            output_ids: receipt.landscape_item_ids().clone(),
            counts,
            warning_count: receipt.warnings().len(),
        })
    }

    /// Certificate the landscape snapshot diff stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn landscape_diff(receipt: &LandscapeDiffReceipt) -> Self {
        let mut counts = BTreeMap::from([
            ("added_items".to_owned(), receipt.counts().added_items()),
            ("changed_items".to_owned(), receipt.counts().changed_items()),
            ("removed_items".to_owned(), receipt.counts().removed_items()),
            ("total_items".to_owned(), receipt.counts().total_items()),
            ("warnings".to_owned(), receipt.warnings().len()),
        ]);
        counts.extend(receipt.subject_kind_counts().clone());
        counts.extend(receipt.change_subject_counts().clone());
        counts.extend(receipt.trace_source_kind_counts().clone());
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::LandscapeDiff,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<LandscapeDiffContract>(),
            requires: contract_requirements::<LandscapeDiffContract>(),
            ensures: contract_guarantees::<LandscapeDiffContract>(),
            input_ids: receipt.source_record_ids().clone(),
            output_ids: vec!["landscape_diff".to_owned()],
            counts,
            warning_count: receipt.warnings().len(),
        })
    }

    /// Certificate the auditor navigation view-model stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn auditor_navigation(receipt: &ViewModelReceipt) -> Self {
        let mut counts = BTreeMap::from([
            (
                "assessment_items".to_owned(),
                receipt.counts().assessment_items(),
            ),
            (
                "assessment_work_items".to_owned(),
                receipt.counts().assessment_work_items(),
            ),
            ("claim_cards".to_owned(), receipt.counts().claim_cards()),
            ("filter_facets".to_owned(), receipt.counts().filter_facets()),
            (
                "backend_coverage_items".to_owned(),
                receipt.counts().backend_coverage_items(),
            ),
            ("gap_items".to_owned(), receipt.counts().gap_items()),
            (
                "proof_chain_items".to_owned(),
                receipt.counts().proof_chain_items(),
            ),
            ("root_items".to_owned(), receipt.counts().root_items()),
            ("source_items".to_owned(), receipt.counts().source_items()),
            ("view_model_items".to_owned(), receipt.view_item_ids().len()),
            ("warnings".to_owned(), receipt.warnings().len()),
            ("witness_items".to_owned(), receipt.counts().witness_items()),
        ]);
        counts.extend(receipt.lane_counts().clone());
        counts.extend(receipt.facet_group_counts().clone());
        counts.extend(receipt.source_artifact_counts().clone());
        counts.extend(receipt.source_kind_counts().clone());
        counts.extend(receipt.proof_chain_topology_counts().clone());
        counts.extend(receipt.provenance_root_kind_counts().clone());
        counts.extend(receipt.review_workflow_counts().clone());
        counts.extend(receipt.trace_kind_counts().clone());
        counts.extend(receipt.trace_source_counts().clone());
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::AuditorNavigation,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<AuditorNavigationContract>(),
            requires: contract_requirements::<AuditorNavigationContract>(),
            ensures: contract_guarantees::<AuditorNavigationContract>(),
            input_ids: receipt.source_record_ids().clone(),
            output_ids: receipt.view_item_ids().clone(),
            counts,
            warning_count: receipt.warnings().len(),
        })
    }

    /// Certificate the `elicit_ui` projection stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn ui_ir_projection(receipt: &UiIrReceipt) -> Self {
        let mut counts = BTreeMap::from([
            (
                "minted_standards".to_owned(),
                receipt.minted_standards().len(),
            ),
            ("sections".to_owned(), receipt.counts().section_count()),
            ("trace_refs".to_owned(), receipt.counts().trace_ref_count()),
            ("widgets".to_owned(), receipt.counts().widget_count()),
            ("tree_nodes".to_owned(), receipt.counts().node_count()),
        ]);
        counts.extend(receipt.role_counts().clone());
        counts.extend(receipt.section_widget_counts().clone());
        counts.extend(receipt.section_population_counts().clone());
        counts.extend(receipt.section_trace_counts().clone());
        counts.extend(receipt.viewport_verification_counts().clone());
        counts.extend(receipt.trace_source_counts().clone());
        counts.extend(receipt.trace_source_kind_counts().clone());
        counts.extend(receipt.minted_prop_counts().clone());
        counts.extend(receipt.proof_token_counts().clone());
        let mut output_ids = receipt.minted_standards().clone();
        output_ids.extend(receipt.section_ids().iter().cloned());
        output_ids.extend(
            receipt
                .viewport_ids()
                .iter()
                .map(|viewport_id| format!("elicit_ui.viewport:{viewport_id}")),
        );
        output_ids.extend(
            receipt
                .section_root_widget_ids()
                .iter()
                .map(|widget_id| format!("elicit_ui.section_root_widget:{widget_id}")),
        );
        output_ids.extend(
            receipt
                .trace_widget_ids()
                .iter()
                .map(|widget_id| format!("elicit_ui.trace_widget:{widget_id}")),
        );
        Self::new(StageCertificateEvidence {
            stage: BelvederePipelineStage::UiIrProjection,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<ElicitUiProjectionContract>(),
            requires: contract_requirements::<ElicitUiProjectionContract>(),
            ensures: contract_guarantees::<ElicitUiProjectionContract>(),
            input_ids: receipt.source_record_ids().clone(),
            output_ids,
            counts,
            warning_count: 0,
        })
    }

    /// Pipeline stage named by this certificate.
    #[must_use]
    pub const fn stage(&self) -> BelvederePipelineStage {
        self.stage
    }

    #[instrument(level = "debug", skip(stage, receipt))]
    fn ingestion<C: BelvedereContract>(
        stage: BelvederePipelineStage,
        receipt: &IngestionReceipt,
    ) -> Self {
        let mut counts = receipt.decoded_counts().clone();
        counts.extend(receipt.source_record_kind_counts().clone());
        counts.extend(receipt.unsupported_field_record_counts().clone());
        counts.insert(
            "unsupported_fields".to_owned(),
            receipt.unsupported_fields().len(),
        );
        counts.insert("warnings".to_owned(), receipt.warnings().len());
        Self::new(StageCertificateEvidence {
            stage,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<C>(),
            requires: contract_requirements::<C>(),
            ensures: contract_guarantees::<C>(),
            input_ids: vec![format!(
                "{}:{}:{}:{}",
                receipt.artifact_kind(),
                receipt.format(),
                receipt.path().display(),
                receipt.digest()
            )],
            output_ids: receipt.source_record_ids().clone(),
            counts,
            warning_count: receipt.warnings().len(),
        })
    }

    #[instrument(level = "debug", skip(evidence))]
    fn new(evidence: StageCertificateEvidence) -> Self {
        let invariant_checks = stage_invariant_checks(
            &evidence.requires,
            &evidence.ensures,
            &evidence.input_ids,
            &evidence.counts,
            evidence.warning_count,
        );
        let digest = stage_digest(&evidence, &invariant_checks);
        let id = format!("belvedere.stage.{}:{digest}", evidence.stage);
        Self {
            schema_id: BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID.to_owned(),
            schema_version: BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
            id,
            stage: evidence.stage,
            contract: evidence.contract,
            standards: evidence.standards,
            requires: evidence.requires,
            ensures: evidence.ensures,
            input_ids: evidence.input_ids,
            output_ids: evidence.output_ids,
            counts: evidence.counts,
            invariant_checks,
            warning_count: evidence.warning_count,
            digest,
        }
    }
}

#[instrument(level = "debug", skip(evidence, invariant_checks))]
fn stage_digest(
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
    update_fields(&mut hasher, &evidence.output_ids);
    for (name, count) in &evidence.counts {
        update_field(&mut hasher, name);
        update_field(&mut hasher, &count.to_string());
    }
    for check in invariant_checks {
        update_field(&mut hasher, check.name());
        update_field(&mut hasher, &check.passed().to_string());
        update_field(&mut hasher, check.detail());
    }
    hex_digest(hasher)
}

#[instrument(level = "trace")]
fn contract_requirements<C: BelvedereContract>() -> Vec<String> {
    C::requires()
        .iter()
        .map(|requirement| (*requirement).to_owned())
        .collect()
}

#[instrument(level = "trace")]
fn contract_standards<C: BelvedereContract>() -> Vec<String> {
    C::standard_ids()
        .iter()
        .map(|standard| (*standard).to_owned())
        .collect()
}

#[instrument(level = "trace")]
fn contract_guarantees<C: BelvedereContract>() -> Vec<String> {
    C::ensures()
        .iter()
        .map(|guarantee| (*guarantee).to_owned())
        .collect()
}
