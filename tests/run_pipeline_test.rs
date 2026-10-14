use std::{
    fs,
    path::{Path, PathBuf},
    process::{self, ExitCode},
};

use belvedere::{
    BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID, BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereCli, BelvedereCliArtifact, BelvedereCliOutput, BelvedereCliOutputSink,
    BelvedereInvocationCommand, BelvedereInvocationSurface, BelvederePipelineStage,
    BelvedereRunInput, DIFF_REVIEW_UI_IR_SECTION_ITEMS,
    DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION, DIFF_REVIEW_UI_IR_SECTION_SUMMARY,
    INPUT_FIDELITY_STANDARD_ID, REPORT_REPRODUCIBILITY_STANDARD_ID,
    UI_IR_SECTION_ASSESSMENT_HISTORY, UI_IR_SECTION_ASSESSMENT_QUEUE,
    UI_IR_SECTION_ASSESSMENT_WORK, UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_DETAIL,
    UI_IR_SECTION_FILTERS, UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_REVIEW_WORKFLOW,
    UI_IR_SECTION_SOURCE_NAVIGATION, VIEW_MODEL_FIDELITY_STANDARD_ID, run_belvedere_pipeline,
    run_cli_with_writer,
};
use miette::{IntoDiagnostic, Result, WrapErr};
use serde_json::Value;

type ReceiptAssertion = fn(&Value);

fn stdio_output(artifact: BelvedereCliArtifact) -> BelvedereCliOutput {
    BelvedereCliOutput::artifact(artifact, BelvedereCliOutputSink::Stdio)
}

fn file_output(artifact: BelvedereCliArtifact, path: PathBuf) -> BelvedereCliOutput {
    BelvedereCliOutput::artifact(artifact, BelvedereCliOutputSink::File(path))
}

fn is_hex_digest(value: &Value) -> bool {
    value.as_str().is_some_and(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
    })
}

#[test]
fn run_pipeline_uses_the_normal_amenable_artifact_path() -> Result<()> {
    belvedere::init_tracing();

    let report = run_belvedere_pipeline(BelvedereRunInput::new(
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ))
    .into_diagnostic()
    .wrap_err("fixture artifacts should drive the normal Belvedere pipeline")?;

    assert_eq!(report.registry_receipt().source_record_ids().len(), 5);
    assert_eq!(report.assessment_receipt().source_record_ids().len(), 2);
    assert_eq!(report.landscape_receipt().claim_count(), 4);
    assert_eq!(report.view_model_receipt().counts().claim_cards(), 4);
    assert_eq!(report.view_model_receipt().counts().filter_facets(), 12);
    assert_eq!(
        report
            .view_model_receipt()
            .counts()
            .backend_coverage_items(),
        12
    );
    assert_eq!(
        report.view_model_receipt().counts().assessment_work_items(),
        2
    );
    assert_eq!(
        report
            .view_model_receipt()
            .lane_counts()
            .get("assessment_work_lane_assessment"),
        Some(&1)
    );
    assert_eq!(report.view_model_receipt().counts().source_items(), 7);
    assert_eq!(report.ui_ir_receipt().counts().section_count(), 14);
    assert!(
        report
            .ui_ir_receipt()
            .section_ids()
            .contains(&UI_IR_SECTION_CLAIM_DETAIL.to_owned())
    );
    assert!(
        report
            .ui_ir_receipt()
            .section_ids()
            .contains(&UI_IR_SECTION_PROOF_CHAINS.to_owned())
    );
    assert!(
        report
            .ui_ir_receipt()
            .section_ids()
            .contains(&UI_IR_SECTION_ASSESSMENT_HISTORY.to_owned())
    );
    assert_eq!(
        report.certificate().schema_id(),
        BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID
    );
    assert_eq!(
        *report.certificate().schema_version(),
        BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION
    );
    assert_eq!(report.certificate().stage_certificates().len(), 5);
    assert_eq!(report.certificate().input_artifact_ids().len(), 2);
    assert!(
        report
            .certificate()
            .input_artifact_ids()
            .iter()
            .any(|artifact_id| artifact_id.starts_with("registry:registry_dump:"))
    );
    assert!(
        report
            .certificate()
            .input_artifact_ids()
            .iter()
            .any(|artifact_id| artifact_id.starts_with("assessment:proof_assessments:"))
    );
    assert_eq!(report.certificate().stage_certificate_ids().len(), 5);
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_registry_ingestion"),
        Some(&1)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_assessment_ingestion"),
        Some(&1)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_landscape_modeling"),
        Some(&1)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_auditor_navigation"),
        Some(&1)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_ui_ir_projection"),
        Some(&1)
    );
    assert_eq!(report.certificate().pipeline_claims().len(), 8);
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_ingestion"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_landscape"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_view_model"),
        Some(&1)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_ui_ir"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_run"),
        Some(&1)
    );
    assert!(
        report
            .certificate()
            .pipeline_claims()
            .iter()
            .any(
                |claim| claim.claim_id() == REPORT_REPRODUCIBILITY_STANDARD_ID
                    && claim.phase() == "run"
            )
    );
    assert!(
        report
            .certificate()
            .pipeline_claims()
            .iter()
            .any(|claim| claim.claim_id() == VIEW_MODEL_FIDELITY_STANDARD_ID
                && claim.phase() == "view_model")
    );
    assert_eq!(
        report.manifest().schema_id(),
        BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID
    );
    assert_eq!(
        *report.manifest().schema_version(),
        BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION
    );
    assert!(
        report
            .manifest()
            .id()
            .starts_with("belvedere.manifest.artifacts:")
    );
    assert_eq!(
        report.manifest().id(),
        &format!(
            "belvedere.manifest.artifacts:{}",
            report.manifest().digest()
        )
    );
    assert_eq!(report.manifest().input_artifacts().len(), 2);
    assert_eq!(report.manifest().stage_certificates().len(), 5);
    assert_eq!(report.manifest().output_artifacts().len(), 35);
    assert!(
        report
            .manifest()
            .invariant_checks()
            .iter()
            .all(|check| check.passed())
    );
    assert_eq!(
        report
            .manifest()
            .input_role_counts()
            .get("manifest_input_role_registry"),
        Some(&1)
    );
    assert_eq!(
        report
            .manifest()
            .stage_kind_counts()
            .get("manifest_stage_kind_ui_ir_projection"),
        Some(&1)
    );
    assert_eq!(
        report
            .manifest()
            .output_kind_counts()
            .get("manifest_output_kind_ui_ir_receipt"),
        Some(&1)
    );
    assert_eq!(
        report
            .manifest()
            .output_kind_counts()
            .get("manifest_output_kind_auditor_backend_coverage"),
        Some(&1)
    );
    assert_eq!(
        report
            .manifest()
            .output_kind_counts()
            .get("manifest_output_kind_elicit_ui_trace_widgets"),
        Some(&1)
    );
    assert_eq!(
        report.manifest().run_certificate_id(),
        report.certificate().id()
    );
    assert!(report.manifest().output_artifacts().iter().any(|artifact| {
        artifact.role() == "ui_ir_projection"
            && artifact.source_stage_role() == "ui_ir_projection"
            && artifact.evidence_id() == report.ui_ir_receipt().receipt_id()
            && artifact.evidence_digest() == report.ui_ir_receipt().receipt_digest()
            && artifact
                .artifact_id()
                .starts_with("belvedere.manifest.output_artifact:")
            && artifact.artifact_id()
                == &format!(
                    "belvedere.manifest.output_artifact:{}",
                    artifact.artifact_digest()
                )
            && *artifact.item_count() == report.ui_ir_receipt().trace_widget_ids().len()
    }));
    assert!(report.manifest().output_artifacts().iter().any(|artifact| {
        artifact.role() == "backend_coverage"
            && artifact.artifact_kind() == "auditor_backend_coverage"
            && artifact.source_stage_role() == "auditor_navigation"
            && artifact.evidence_id() == report.view_model_receipt().receipt_id()
            && artifact.evidence_digest() == report.view_model_receipt().receipt_digest()
            && *artifact.item_count()
                == report
                    .view_model_receipt()
                    .counts()
                    .backend_coverage_items()
    }));
    assert!(report.manifest().output_artifacts().iter().any(|artifact| {
        artifact.role() == "ui_ir_trace_widgets"
            && artifact.artifact_kind() == "elicit_ui_trace_widgets"
            && artifact.source_stage_role() == "ui_ir_projection"
            && artifact.evidence_id() == report.ui_ir_receipt().receipt_id()
            && artifact.evidence_digest() == report.ui_ir_receipt().receipt_digest()
            && *artifact.item_count() == report.ui_ir_receipt().trace_widget_ids().len()
    }));
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .all(
                |certificate| certificate.schema_id() == BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID
                    && *certificate.schema_version() == BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION
                    && !certificate.standards().is_empty()
                    && !certificate.requires().is_empty()
                    && !certificate.ensures().is_empty()
            )
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .standards()
                .iter()
                .any(|standard| standard == INPUT_FIDELITY_STANDARD_ID))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .filter(|certificate| {
                matches!(
                    certificate.stage(),
                    BelvederePipelineStage::RegistryIngestion
                        | BelvederePipelineStage::AssessmentIngestion
                )
            })
            .all(
                |certificate| certificate.invariant_checks().iter().any(|check| {
                    check.name() == "ingestion_source_record_ids_classified" && check.passed()
                })
            )
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .filter(|certificate| {
                matches!(
                    certificate.stage(),
                    BelvederePipelineStage::RegistryIngestion
                        | BelvederePipelineStage::AssessmentIngestion
                )
            })
            .all(
                |certificate| certificate.invariant_checks().iter().any(|check| {
                    check.name() == "ingestion_decoded_counts_match_source_records"
                        && check.passed()
                })
            )
    );
    assert!(
        report
            .certificate()
            .invariant_checks()
            .iter()
            .all(|check| check.passed())
    );
    assert!(
        report
            .certificate()
            .invariant_checks()
            .iter()
            .any(|check| check.name() == "run_stage_certificate_digests_match")
    );
    assert!(
        report
            .certificate()
            .invariant_checks()
            .iter()
            .any(|check| check.name() == "run_stage_certificate_kinds_classified")
    );
    assert!(
        report
            .certificate()
            .invariant_checks()
            .iter()
            .any(|check| check.name() == "run_pipeline_claims_recorded" && check.passed())
    );
    assert!(
        report
            .certificate()
            .invariant_checks()
            .iter()
            .any(|check| check.name() == "run_pipeline_claim_phases_classified" && check.passed())
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("ui_ir_proof_token_viewport_wcag_verified")
                == Some(&3))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("gap_lane_witness_review") == Some(&1))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(
                |certificate| certificate.counts().get("landscape_gap_kind_MissingProof")
                    == Some(&2)
            )
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("assessment_recency_current") == Some(&2))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(
                |certificate| certificate.counts().get("source_artifact_registry_dump") == Some(&5)
            )
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("source_kind_assessment") == Some(&2))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("source_kind_kani_proof") == Some(&1))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("facet_group_gap_kind") == Some(&3))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| {
                certificate
                    .counts()
                    .get("facet_group_assessment_queue_group")
                    == Some(&2)
            })
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("proof_chain_max_depth") == Some(&1))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("provenance_root_kind_assessment_only_root")
                == Some(&1))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("review_workflow_proof_chain_state_blocked")
                == Some(&2))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("review_workflow_task_total") == Some(&4))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("review_workflow_assessment_work_total")
                == Some(&2))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("review_workflow_gap_high_priority")
                == Some(&3))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("view_model_trace_kind_backend_coverage_item")
                == Some(&12))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("view_model_trace_unique_source_records")
                == Some(&7))
    );
    let ui_ir_assessment_trace_links = *report
        .ui_ir_receipt()
        .trace_source_kind_counts()
        .get("trace_source_kind_assessment")
        .wrap_err("UI IR receipt should count assessment source trace links")?;
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(
                |certificate| certificate.counts().get("trace_source_kind_assessment")
                    == Some(&ui_ir_assessment_trace_links)
            )
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("role_button") == Some(&27))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .counts()
                .get("minted_elicit_ui_prop_WcagVerified")
                == Some(&1))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(
                |certificate| certificate.counts().get("ui_ir_proof_token_wcag_verified")
                    == Some(&1)
            )
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("trace_unique_source_records") == Some(&7))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(
                |certificate| certificate.counts().get(UI_IR_SECTION_SOURCE_NAVIGATION) == Some(&7)
            )
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .output_ids()
                .contains(&UI_IR_SECTION_FILTERS.to_owned()))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .output_ids()
                .contains(&UI_IR_SECTION_PROOF_CHAINS.to_owned()))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .output_ids()
                .contains(&UI_IR_SECTION_REVIEW_WORKFLOW.to_owned()))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .output_ids()
                .contains(&UI_IR_SECTION_BACKEND_COVERAGE.to_owned()))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate
                .output_ids()
                .contains(&UI_IR_SECTION_SOURCE_NAVIGATION.to_owned()))
    );
    let landscape_stage = report
        .certificate()
        .stage_certificates()
        .iter()
        .find(|certificate| certificate.stage() == BelvederePipelineStage::LandscapeModeling)
        .ok_or_else(|| {
            miette::miette!("run certificate should include landscape modeling stage evidence")
        })?;
    assert_eq!(
        landscape_stage.counts().get("landscape_items"),
        Some(&report.landscape_receipt().landscape_item_ids().len())
    );
    assert_eq!(
        landscape_stage.counts().get("source_receipts"),
        Some(&report.landscape_receipt().source_receipt_ids().len())
    );
    assert_eq!(
        landscape_stage.counts().get("landscape_item_kind_claim"),
        Some(&report.landscape_receipt().claim_count())
    );
    assert!(
        landscape_stage
            .invariant_checks()
            .iter()
            .any(|check| check.name() == "landscape_item_ids_classified" && check.passed())
    );
    assert!(
        landscape_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "landscape_core_counts_match_items" && check.passed() })
    );
    assert!(
        report
            .landscape_receipt()
            .landscape_item_ids()
            .iter()
            .all(|landscape_item_id| landscape_stage.output_ids().contains(landscape_item_id))
    );
    assert_eq!(
        landscape_stage.input_ids(),
        report.landscape_receipt().source_receipt_ids()
    );
    assert_eq!(
        landscape_stage.input_digests(),
        report.landscape_receipt().source_receipt_digests()
    );
    let auditor_stage = report
        .certificate()
        .stage_certificates()
        .iter()
        .find(|certificate| certificate.stage() == BelvederePipelineStage::AuditorNavigation)
        .ok_or_else(|| {
            miette::miette!("run certificate should include auditor navigation stage evidence")
        })?;
    assert_eq!(
        auditor_stage.counts().get("view_model_items"),
        Some(&report.view_model_receipt().view_item_ids().len())
    );
    assert_eq!(
        auditor_stage.counts().get("view_model_item_kind_claim"),
        Some(&report.view_model_receipt().counts().claim_cards())
    );
    assert_eq!(
        auditor_stage.counts().get("claim_detail_items"),
        Some(&report.view_model_receipt().counts().claim_detail_items())
    );
    assert_eq!(
        auditor_stage.counts().get("review_workflow_items"),
        Some(&report.view_model_receipt().counts().review_workflow_items())
    );
    assert_eq!(
        auditor_stage.counts().get("view_model_item_kind_gap"),
        Some(&report.view_model_receipt().counts().gap_items())
    );
    assert_eq!(auditor_stage.counts().get("source_landscapes"), Some(&1));
    assert_eq!(
        auditor_stage.counts().get("source_locator_items"),
        Some(&report.view_model_receipt().source_record_ids().len())
    );
    assert_eq!(
        auditor_stage.input_ids(),
        &vec![report.landscape_receipt().receipt_id().to_owned()]
    );
    assert_eq!(
        auditor_stage.input_digests(),
        &vec![report.landscape_receipt().receipt_digest().to_owned()]
    );
    assert!(
        report
            .view_model_receipt()
            .view_item_ids()
            .iter()
            .all(|view_item_id| auditor_stage.output_ids().contains(view_item_id))
    );
    assert!(
        auditor_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "view_model_trace_sources_complete" && check.passed() })
    );
    assert!(
        auditor_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "view_model_item_ids_classified" && check.passed() })
    );
    assert!(
        auditor_stage.invariant_checks().iter().any(|check| {
            check.name() == "view_model_core_counts_match_items" && check.passed()
        })
    );
    assert!(
        auditor_stage.invariant_checks().iter().any(|check| {
            check.name() == "view_model_input_sources_reachable" && check.passed()
        })
    );
    assert!(
        auditor_stage.invariant_checks().iter().any(|check| {
            check.name() == "view_model_source_locators_complete" && check.passed()
        })
    );
    let ui_stage = report
        .certificate()
        .stage_certificates()
        .iter()
        .find(|certificate| certificate.stage() == BelvederePipelineStage::UiIrProjection)
        .ok_or_else(|| {
            miette::miette!("run certificate should include UI IR projection stage evidence")
        })?;
    assert_eq!(ui_stage.counts().get("ui_ir_sections_populated"), Some(&14));
    assert_eq!(ui_stage.counts().get("ui_ir_sections_empty"), Some(&0));
    assert_eq!(
        ui_stage.counts().get("ui_ir_section_widgets_total"),
        Some(&report.projection().trace_widgets().len())
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_sections_with_source_widgets"),
        Some(&14)
    );
    assert_eq!(
        ui_stage
            .counts()
            .get("ui_ir_section_trace_source_links_total"),
        report
            .ui_ir_receipt()
            .trace_source_counts()
            .get("trace_source_links")
    );
    assert_eq!(
        ui_stage
            .counts()
            .get("ui_ir_section_trace_widgets:belvedere.ui.section.source_navigation"),
        Some(&7)
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_source_navigation_items"),
        Some(&7)
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_source_locator_items"),
        Some(&7)
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_source_locator_widgets"),
        Some(&7)
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_viewports_verified_total"),
        Some(&3)
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_output_kind_minted_standard"),
        Some(&report.projection().minted_standards().len())
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_output_kind_section"),
        Some(&report.projection().sections().len())
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_output_kind_viewport"),
        Some(&report.projection().viewport_verifications().len())
    );
    assert_eq!(
        ui_stage
            .counts()
            .get("ui_ir_output_kind_section_root_widget"),
        Some(&report.projection().sections().len())
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_output_kind_trace_widget"),
        Some(&report.projection().trace_widgets().len())
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_output_kind_trace_widget_ref"),
        Some(&report.projection().trace_widgets().len())
    );
    assert_eq!(ui_stage.counts().get("source_view_models"), Some(&1));
    assert_eq!(
        ui_stage.input_ids(),
        &vec![report.view_model_receipt().receipt_id().to_owned()]
    );
    assert_eq!(
        ui_stage.input_digests(),
        &vec![report.view_model_receipt().receipt_digest().to_owned()]
    );
    assert_eq!(ui_stage.output_ids(), report.ui_ir_receipt().output_ids());
    assert!(
        report
            .ui_ir_receipt()
            .viewport_ids()
            .iter()
            .all(|viewport_id| ui_stage
                .output_ids()
                .contains(&format!("elicit_ui.viewport:{viewport_id}")))
    );
    assert!(
        report
            .ui_ir_receipt()
            .section_root_widget_ids()
            .iter()
            .all(|widget_id| ui_stage
                .output_ids()
                .contains(&format!("elicit_ui.section_root_widget:{widget_id}")))
    );
    assert!(
        report
            .ui_ir_receipt()
            .trace_widget_ids()
            .iter()
            .all(|widget_id| ui_stage
                .output_ids()
                .contains(&format!("elicit_ui.trace_widget:{widget_id}")))
    );
    assert!(
        report
            .ui_ir_receipt()
            .trace_widget_ref_ids()
            .iter()
            .all(|trace_ref_id| ui_stage.output_ids().contains(trace_ref_id))
    );
    assert!(
        ui_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "ui_ir_trace_widgets_sourced" && check.passed() })
    );
    assert!(
        ui_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "ui_ir_trace_widgets_sectioned" && check.passed() })
    );
    assert!(
        ui_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "ui_ir_sections_traceable" && check.passed() })
    );
    assert!(
        ui_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "ui_ir_input_sources_reachable" && check.passed() })
    );
    assert!(
        ui_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "ui_ir_output_ids_classified" && check.passed() })
    );
    assert!(
        ui_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "ui_ir_core_counts_match_outputs" && check.passed() })
    );
    assert!(
        ui_stage
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "ui_ir_source_locators_preserved" && check.passed() })
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .flat_map(|certificate| certificate.invariant_checks())
            .all(|check| check.passed())
    );
    assert!(report.projection().trace_widgets().len() >= 4);
    Ok(())
}

#[test]
fn run_certificate_is_reproducible_portable_evidence() -> Result<()> {
    belvedere::init_tracing();

    let first = run_belvedere_pipeline(BelvedereRunInput::new(
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ))
    .into_diagnostic()
    .wrap_err("first fixture pipeline run should produce a certificate")?;
    let second = run_belvedere_pipeline(BelvedereRunInput::new(
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ))
    .into_diagnostic()
    .wrap_err("second fixture pipeline run should produce a certificate")?;

    assert_eq!(first.certificate().id(), second.certificate().id());
    assert_eq!(first.manifest().id(), second.manifest().id());
    assert_eq!(first.manifest().digest(), second.manifest().digest());
    assert!(
        first
            .manifest()
            .id()
            .starts_with("belvedere.manifest.artifacts:")
    );
    assert_eq!(
        first.certificate().invocation().surface(),
        BelvedereInvocationSurface::Library
    );
    assert_eq!(first.certificate().invocation().command(), None);
    assert_eq!(first.certificate().invocation().output_artifact(), &None);
    assert_eq!(first.certificate().invocation().output_sink(), &None);
    assert_eq!(
        first
            .certificate()
            .component_versions()
            .get("belvedere")
            .map(String::as_str),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        first
            .certificate()
            .component_versions()
            .get("amenable")
            .map(String::as_str),
        Some("0.1.0")
    );
    assert_eq!(
        first
            .certificate()
            .component_versions()
            .get("elicitation")
            .map(String::as_str),
        Some("0.11.1")
    );
    assert_eq!(
        first
            .certificate()
            .component_versions()
            .get("elicit_ui")
            .map(String::as_str),
        Some("0.11.1")
    );
    assert_eq!(
        first.certificate().stage_certificate_ids(),
        second.certificate().stage_certificate_ids()
    );
    assert_eq!(
        first.certificate().input_artifact_digests(),
        second.certificate().input_artifact_digests()
    );
    let expected_input_artifact_digests = vec![
        first.registry_receipt().digest().to_owned(),
        first.assessment_receipt().digest().to_owned(),
    ];
    assert_eq!(
        first.certificate().input_artifact_digests(),
        &expected_input_artifact_digests
    );
    assert_eq!(
        first.certificate().stage_certificate_digests(),
        second.certificate().stage_certificate_digests()
    );
    assert_eq!(first.certificate().stage_certificate_digests().len(), 5);
    assert!(
        first
            .certificate()
            .stage_certificates()
            .iter()
            .zip(first.certificate().stage_certificate_digests())
            .all(|(certificate, digest)| certificate.digest() == digest)
    );

    let encoded = serde_json::to_string(first.certificate())
        .into_diagnostic()
        .wrap_err("run certificate should serialize as portable JSON evidence")?;
    assert!(encoded.contains(BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID));
    assert!(encoded.contains(BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID));
    assert!(encoded.contains("belvedere.run:"));
    assert!(encoded.contains("ui_ir_projection"));
    assert!(encoded.contains("source record identities remain available for traceability"));
    assert!(encoded.contains("the emitted tree is an elicit_ui VerifiedTree"));
    assert!(encoded.contains("contract_obligations_named"));
    assert!(encoded.contains("run_stage_certificate_digests_match"));
    assert!(encoded.contains("run_stage_certificate_kinds_classified"));
    assert!(encoded.contains("run_cli_contract_recorded"));
    assert!(encoded.contains("run_pipeline_claim_phases_classified"));
    assert!(encoded.contains("pipeline_claim_phase_counts"));
    assert!(encoded.contains("stage_kind_counts"));
    assert!(encoded.contains("warning_count_consistent"));
    Ok(())
}

#[test]
fn cli_help_is_a_successful_public_output_contract() -> Result<()> {
    belvedere::init_tracing();

    let args = ["--help"]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("help should be part of the public CLI API")?;

    assert_eq!(command.output(), BelvedereCliOutput::Help);

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let help = String::from_utf8(output)
        .into_diagnostic()
        .wrap_err("help output should be UTF-8")?;
    let diff_help = help
        .split_once("Diff artifacts:")
        .map(|(_heading, section)| section)
        .ok_or_else(|| miette::miette!("help output should include a diff artifacts section"))?;
    assert!(help.contains("usage: belvedere"));
    assert!(help.contains("belvedere diff"));
    assert!(help.contains("--stdio"));
    assert!(help.contains("--file <path>"));
    assert!(help.contains("--certificate"));
    assert!(help.contains("--registry-ingestion-certificate"));
    assert!(help.contains("--assessment-ingestion-certificate"));
    assert!(help.contains("--landscape-modeling-certificate"));
    assert!(help.contains("--auditor-navigation-certificate"));
    assert!(help.contains("--ui-ir-projection-certificate"));
    assert!(help.contains("--manifest"));
    assert!(help.contains("--run-certificate"));
    assert!(help.contains("--diff-review-certificate"));
    assert!(help.contains("--diff-review-ui-ir-certificate"));
    assert!(help.contains("--diff"));
    assert!(help.contains("--diff-receipt"));
    assert!(help.contains("--previous-registry-ingestion-certificate"));
    assert!(help.contains("--previous-assessment-ingestion-certificate"));
    assert!(help.contains("--previous-landscape-modeling-certificate"));
    assert!(help.contains("--previous-auditor-navigation-certificate"));
    assert!(help.contains("--previous-ui-ir-projection-certificate"));
    assert!(help.contains("--current-registry-ingestion-certificate"));
    assert!(help.contains("--current-assessment-ingestion-certificate"));
    assert!(help.contains("--current-landscape-modeling-certificate"));
    assert!(help.contains("--current-auditor-navigation-certificate"));
    assert!(help.contains("--current-ui-ir-projection-certificate"));
    assert!(help.contains("--previous-registry-receipt"));
    assert!(help.contains("--previous-assessment-receipt"));
    assert!(help.contains("--current-registry-receipt"));
    assert!(help.contains("--current-assessment-receipt"));
    assert!(help.contains("--previous-landscape-receipt"));
    assert!(help.contains("--previous-view-model"));
    assert!(help.contains("--previous-overview"));
    assert!(help.contains("--previous-backend-coverage"));
    assert!(help.contains("--previous-claim-explorer"));
    assert!(help.contains("--previous-claim-details"));
    assert!(help.contains("--previous-filter-facets"));
    assert!(help.contains("--previous-source-navigation"));
    assert!(help.contains("--previous-proof-chains"));
    assert!(help.contains("--previous-review-workflow"));
    assert!(help.contains("--previous-assessment-history"));
    assert!(help.contains("--previous-assessment-queue"));
    assert!(help.contains("--previous-assessment-work"));
    assert!(help.contains("--previous-witness-surface"));
    assert!(help.contains("--previous-provenance-roots"));
    assert!(help.contains("--previous-gaps"));
    assert!(help.contains("--previous-view-model-receipt"));
    assert!(help.contains("--previous-ui-ir"));
    assert!(help.contains("--previous-ui-ir-sections"));
    assert!(help.contains("--previous-ui-ir-trace-widgets"));
    assert!(help.contains("--previous-ui-ir-viewports"));
    assert!(help.contains("--previous-ui-ir-minted-standards"));
    assert!(help.contains("--previous-ui-ir-receipt"));
    assert!(help.contains("--current-landscape-receipt"));
    assert!(help.contains("--current-overview"));
    assert!(help.contains("--current-backend-coverage"));
    assert!(help.contains("--current-claim-explorer"));
    assert!(help.contains("--current-claim-details"));
    assert!(help.contains("--current-filter-facets"));
    assert!(help.contains("--current-source-navigation"));
    assert!(help.contains("--current-proof-chains"));
    assert!(help.contains("--current-review-workflow"));
    assert!(help.contains("--current-assessment-history"));
    assert!(help.contains("--current-assessment-queue"));
    assert!(help.contains("--current-assessment-work"));
    assert!(help.contains("--current-witness-surface"));
    assert!(help.contains("--current-provenance-roots"));
    assert!(help.contains("--current-gaps"));
    assert!(help.contains("--current-view-model-receipt"));
    assert!(help.contains("--current-ui-ir-sections"));
    assert!(help.contains("--current-ui-ir-trace-widgets"));
    assert!(help.contains("--current-ui-ir-viewports"));
    assert!(help.contains("--current-ui-ir-minted-standards"));
    assert!(help.contains("--current-ui-ir-receipt"));
    assert!(help.contains("--registry-receipt"));
    assert!(help.contains("--assessment-receipt"));
    assert!(help.contains("--landscape-receipt"));
    assert!(help.contains("--overview"));
    assert!(help.contains("--backend-coverage"));
    assert!(help.contains("--claim-explorer"));
    assert!(help.contains("--claim-details"));
    assert!(help.contains("--filter-facets"));
    assert!(help.contains("--source-navigation"));
    assert!(help.contains("--proof-chains"));
    assert!(help.contains("--view-model-receipt"));
    assert!(help.contains("--review-workflow"));
    assert!(help.contains("--assessment-history"));
    assert!(help.contains("--assessment-queue"));
    assert!(help.contains("--assessment-work"));
    assert!(help.contains("--witness-surface"));
    assert!(help.contains("--provenance-roots"));
    assert!(help.contains("--gaps"));
    assert!(help.contains("--ui-ir-sections"));
    assert!(help.contains("--ui-ir-trace-widgets"));
    assert!(help.contains("--ui-ir-viewports"));
    assert!(help.contains("--ui-ir-minted-standards"));
    assert!(help.contains("--ui-ir-receipt"));
    for flag in [
        "--landscape",
        "--landscape-receipt",
        "--view-model",
        "--overview",
        "--backend-coverage",
        "--claim-explorer",
        "--claim-details",
        "--filter-facets",
        "--source-navigation",
        "--proof-chains",
        "--review-workflow",
        "--assessment-history",
        "--assessment-queue",
        "--assessment-work",
        "--witness-surface",
        "--provenance-roots",
        "--gaps",
        "--view-model-receipt",
        "--ui-ir",
        "--ui-ir-sections",
        "--ui-ir-trace-widgets",
        "--ui-ir-viewports",
        "--ui-ir-minted-standards",
        "--ui-ir-receipt",
    ] {
        assert!(
            help_section_lists_flag(diff_help, flag),
            "diff help should list current snapshot alias {flag}"
        );
    }
    assert!(!help.contains("--certificate-json"));
    assert!(!help.contains("--certificate-file"));
    Ok(())
}

fn help_section_lists_flag(section: &str, flag: &str) -> bool {
    section.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed
            .strip_prefix(flag)
            .is_some_and(|rest| rest.chars().next().is_none_or(char::is_whitespace))
    })
}

fn assert_registry_ingestion_receipt(receipt: &Value) {
    assert_ingestion_receipt_identity(receipt);
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ingest.registry_dump")
    );
    assert_eq!(receipt["artifact_kind"].as_str(), Some("registry_dump"));
    assert_eq!(
        receipt["format"].as_str(),
        Some("amenable.registry_dump_json")
    );
    assert_eq!(
        receipt["decoded_counts"]["evidence_links"].as_u64(),
        Some(1)
    );
    assert_eq!(receipt["decoded_counts"]["proof_records"].as_u64(), Some(1));
    assert_eq!(
        receipt["decoded_counts"]["contract_records"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["decoded_counts"]["witness_export_records"].as_u64(),
        Some(1)
    );
    assert_eq!(receipt["decoded_counts"]["kani_proofs"].as_u64(), Some(1));
    assert_eq!(
        receipt["source_record_kind_counts"]["source_record_evidence_link"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(5)
    );
    assert!(receipt["source_record_ids"].as_array().is_some_and(|ids| {
        ids.iter().any(|id| {
            id.as_str() == Some("evidence_link:belvedere::ingestion::DecodedRegistrySnapshot#0")
        })
    }));
}

fn assert_assessment_ingestion_receipt(receipt: &Value) {
    assert_ingestion_receipt_identity(receipt);
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ingest.proof_assessments")
    );
    assert_eq!(receipt["artifact_kind"].as_str(), Some("proof_assessments"));
    assert_eq!(
        receipt["format"].as_str(),
        Some("amenable.proof_assessment_jsonl")
    );
    assert_eq!(receipt["decoded_counts"]["assessments"].as_u64(), Some(2));
    assert_eq!(
        receipt["source_record_kind_counts"]["source_record_assessment"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(2)
    );
    assert!(receipt["source_record_ids"].as_array().is_some_and(|ids| {
        ids.iter()
            .any(|id| id.as_str() == Some("assessment:assess-belvedere-ingestion-1"))
    }));
}

fn assert_empty_assessment_ingestion_receipt(receipt: &Value) {
    assert_ingestion_receipt_identity(receipt);
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ingest.proof_assessments")
    );
    assert_eq!(receipt["artifact_kind"].as_str(), Some("proof_assessments"));
    assert_eq!(
        receipt["format"].as_str(),
        Some("amenable.proof_assessment_jsonl")
    );
    assert_eq!(receipt["decoded_counts"]["assessments"].as_u64(), Some(0));
    assert_eq!(
        receipt["source_record_kind_counts"]
            .as_object()
            .map(serde_json::Map::len),
        Some(0)
    );
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(0)
    );
}

fn assert_ingestion_receipt_identity(receipt: &Value) {
    assert!(
        receipt["receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.ingestion:"))
    );
    assert_eq!(
        receipt["receipt_id"].as_str().map(str::to_owned),
        receipt["receipt_digest"]
            .as_str()
            .map(|digest| format!("belvedere.receipt.ingestion:{digest}"))
    );
}

fn diff_args_with_empty_previous_assessments(
    output_args: &[&str],
    previous_assessments_path: &Path,
) -> Result<Vec<String>> {
    let previous_assessments_path = previous_assessments_path
        .to_str()
        .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?;
    let output_args = match output_args {
        [artifact] => vec![*artifact, "--stdio"],
        [artifact, path] => vec![*artifact, "--file", *path],
        _ => output_args.to_vec(),
    };
    Ok(["diff"]
        .into_iter()
        .chain(output_args)
        .chain([
            "tests/fixtures/registry_dump.json",
            previous_assessments_path,
            "tests/fixtures/registry_dump.json",
            "tests/fixtures/proof_assessments.jsonl",
        ])
        .map(str::to_owned)
        .collect())
}

fn assert_landscape_receipt(receipt: &Value) {
    assert_landscape_receipt_identity(receipt);
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.model.landscape")
    );
    assert_eq!(receipt["counts"]["claim_count"].as_u64(), Some(4));
    assert_eq!(receipt["counts"]["proof_count"].as_u64(), Some(2));
    assert_eq!(receipt["counts"]["gap_count"].as_u64(), Some(4));
    assert_eq!(
        receipt["gap_kind_counts"]["landscape_gap_kind_MissingProof"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["item_kind_counts"]["landscape_item_kind_claim"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["item_kind_counts"]["landscape_item_kind_contract"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["item_kind_counts"]["landscape_item_kind_witness"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["item_kind_counts"]["landscape_item_kind_gap"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["assessment_recency_counts"]["assessment_recency_current"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["landscape_item_ids"].as_array().map(Vec::len),
        Some(19)
    );
    assert!(receipt["landscape_item_ids"].as_array().is_some_and(|ids| {
        ids.iter()
            .any(|id| id.as_str() == Some("landscape.assessment:assess-belvedere-ingestion-1"))
    }));
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(7)
    );
    assert_landscape_receipt_sources(receipt, 2);
}

fn assert_previous_landscape_receipt(receipt: &Value) {
    assert_landscape_receipt_identity(receipt);
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.model.landscape")
    );
    assert_eq!(receipt["counts"]["claim_count"].as_u64(), Some(3));
    assert_eq!(receipt["counts"]["proof_count"].as_u64(), Some(2));
    assert_eq!(
        receipt["assessment_recency_counts"]
            .as_object()
            .map(serde_json::Map::len),
        Some(0)
    );
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(5)
    );
    assert_landscape_receipt_sources(receipt, 2);
}

fn assert_landscape_receipt_identity(receipt: &Value) {
    assert!(
        receipt["receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.landscape:"))
    );
    assert_eq!(
        receipt["receipt_id"].as_str().map(str::to_owned),
        receipt["receipt_digest"]
            .as_str()
            .map(|digest| format!("belvedere.receipt.landscape:{digest}"))
    );
}

fn assert_landscape_receipt_sources(receipt: &Value, expected_count: usize) {
    assert_eq!(
        receipt["source_receipt_ids"].as_array().map(Vec::len),
        Some(expected_count)
    );
    assert!(receipt["source_receipt_ids"].as_array().is_some_and(|ids| {
        ids.iter().all(|id| {
            id.as_str()
                .is_some_and(|id| id.starts_with("belvedere.receipt.ingestion:"))
        })
    }));
    assert_eq!(
        receipt["source_receipt_digests"].as_array().map(Vec::len),
        Some(expected_count)
    );
    assert!(
        receipt["source_receipt_digests"]
            .as_array()
            .is_some_and(|digests| digests.iter().all(is_hex_digest))
    );
}

fn assert_landscape_graph(graph: &Value) {
    assert_eq!(
        graph["claims"].as_object().map(serde_json::Map::len),
        Some(4)
    );
    assert_eq!(graph["edges"].as_array().map(Vec::len), Some(1));
    assert_eq!(graph["assessments"].as_array().map(Vec::len), Some(2));
    assert_eq!(graph["gaps"].as_array().map(Vec::len), Some(4));
    assert_eq!(graph["source_record_ids"].as_array().map(Vec::len), Some(7));
    assert!(graph["claims"].as_object().is_some_and(|claims| {
        claims.contains_key("belvedere::ingestion::DecodedRegistrySnapshot")
    }));
    assert!(graph["assessments"].as_array().is_some_and(|assessments| {
        assessments.iter().any(|assessment| {
            assessment["id"].as_str() == Some("assess-belvedere-ingestion-1")
                && assessment["current"].as_bool() == Some(true)
        })
    }));
    assert!(graph["gaps"].as_array().is_some_and(|gaps| {
        gaps.iter()
            .any(|gap| gap["kind"].as_str() == Some("missing_proof"))
    }));
}

fn assert_previous_landscape_graph(graph: &Value) {
    assert_eq!(
        graph["claims"].as_object().map(serde_json::Map::len),
        Some(3)
    );
    assert_eq!(graph["edges"].as_array().map(Vec::len), Some(1));
    assert_eq!(graph["assessments"].as_array().map(Vec::len), Some(0));
    assert_eq!(graph["source_record_ids"].as_array().map(Vec::len), Some(5));
    assert!(graph["claims"].as_object().is_some_and(|claims| {
        claims.contains_key("belvedere::ingestion::record_identity_is_preserved")
    }));
    assert!(graph["gaps"].as_array().is_some_and(|gaps| {
        gaps.iter()
            .any(|gap| gap["kind"].as_str() == Some("missing_assessment"))
    }));
}

fn assert_landscape_diff_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.diff.landscape")
    );
    assert_landscape_diff_receipt_identity(receipt);
    assert_eq!(receipt["counts"]["added_items"].as_u64(), Some(4));
    assert_eq!(receipt["counts"]["removed_items"].as_u64(), Some(1));
    assert_eq!(receipt["counts"]["changed_items"].as_u64(), Some(1));
    assert_eq!(
        receipt["subject_kind_counts"]["diff_subject_assessment"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["item_kind_counts"]["diff_item_kind_added"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["item_kind_counts"]["diff_item_kind_changed"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["item_kind_counts"]["diff_item_kind_removed"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["change_subject_counts"]["diff_change_added_assessment"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["review_priority_counts"]["diff_review_priority_review"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["review_priority_counts"]["diff_review_priority_blocking"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["review_action_counts"]["diff_review_action_review_new_assessment"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["review_action_counts"]["diff_review_action_triage_new_gap"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["trace_source_kind_counts"]["diff_trace_source_kind_assessment"].as_u64(),
        Some(5)
    );
    assert_eq!(
        receipt["trace_source_counts"]["diff_trace_source_links"].as_u64(),
        Some(8)
    );
    assert_eq!(
        receipt["trace_source_counts"]["diff_trace_unique_source_records"].as_u64(),
        Some(3)
    );
    assert!(
        receipt["trace_source_record_ids"]
            .as_array()
            .is_some_and(|ids| ids
                .iter()
                .any(|id| id.as_str() == Some("assessment:assess-belvedere-ingestion-1")))
    );
    assert_eq!(receipt["diff_item_ids"].as_array().map(Vec::len), Some(6));
    assert!(receipt["diff_item_ids"].as_array().is_some_and(|ids| {
        ids.iter().any(|id| {
            id.as_str() == Some("belvedere.diff.Added.assessment:assess-belvedere-ingestion-1")
        })
    }));
    assert_eq!(
        receipt["source_landscape_receipt_ids"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert!(
        receipt["source_landscape_receipt_ids"]
            .as_array()
            .is_some_and(|ids| ids.iter().all(|id| id
                .as_str()
                .is_some_and(|id| id.starts_with("belvedere.receipt.landscape:"))))
    );
    assert_eq!(
        receipt["source_landscape_receipt_digests"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert!(
        receipt["source_landscape_receipt_digests"]
            .as_array()
            .is_some_and(|digests| digests.iter().all(is_hex_digest))
    );
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(7)
    );
}

fn assert_landscape_diff_receipt_identity(receipt: &Value) {
    assert!(
        receipt["receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.landscape_diff:"))
    );
    assert_eq!(
        receipt["receipt_id"].as_str().map(str::to_owned),
        receipt["receipt_digest"]
            .as_str()
            .map(|digest| format!("belvedere.receipt.landscape_diff:{digest}"))
    );
}

fn assert_auditor_navigation_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.view_model.auditor_navigation")
    );
    assert_view_model_receipt_identity(receipt);
    assert_auditor_navigation_receipt_source(receipt);
    assert_eq!(receipt["counts"]["claim_cards"].as_u64(), Some(4));
    assert_eq!(receipt["counts"]["claim_detail_items"].as_u64(), Some(4));
    assert_eq!(
        receipt["source_kind_counts"]["source_kind_assessment"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["source_locator_counts"]["source_locator_items"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["source_locator_counts"]["source_locator_registry_dump"].as_u64(),
        Some(5)
    );
    assert_eq!(
        receipt["source_locator_counts"]["source_locator_proof_assessments"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["trace_kind_counts"]["view_model_trace_kind_backend_coverage_item"].as_u64(),
        Some(12)
    );
    assert_eq!(
        receipt["trace_kind_counts"]["view_model_trace_kind_claim_detail_item"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["trace_kind_counts"]["view_model_trace_kind_assessment_history_item"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["counts"]["assessment_history_items"].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["trace_source_counts"]["view_model_trace_unique_source_records"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["item_kind_counts"]["view_model_item_kind_overview"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["item_kind_counts"]["view_model_item_kind_claim"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["item_kind_counts"]["view_model_item_kind_backend_coverage"].as_u64(),
        Some(12)
    );
    assert_eq!(
        receipt["item_kind_counts"]["view_model_item_kind_gap"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["item_kind_counts"]["view_model_item_kind_source"].as_u64(),
        Some(7)
    );
    assert_eq!(receipt["view_item_ids"].as_array().map(Vec::len), Some(62));
    assert_eq!(
        receipt["review_workflow_task_digests"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert!(
        receipt["review_workflow_task_digests"]
            .as_array()
            .is_some_and(|digests| digests.iter().all(is_hex_digest))
    );
    assert!(receipt["view_item_ids"].as_array().is_some_and(|ids| {
        ids.iter().any(|id| {
            id.as_str() == Some("view_model.claim:belvedere::ingestion::DecodedRegistrySnapshot")
        })
    }));
    assert!(receipt["view_item_ids"].as_array().is_some_and(|ids| {
        ids.iter().any(|id| {
            id.as_str()
                == Some(
                    "view_model.assessment_history:belvedere::ingestion::record_identity_is_preserved",
                )
        })
    }));
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(7)
    );
}

fn assert_previous_auditor_navigation_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.view_model.auditor_navigation")
    );
    assert_view_model_receipt_identity(receipt);
    assert_auditor_navigation_receipt_source(receipt);
    assert_eq!(receipt["counts"]["claim_cards"].as_u64(), Some(3));
    assert_eq!(receipt["counts"]["claim_detail_items"].as_u64(), Some(3));
    assert_eq!(receipt["counts"]["assessment_items"].as_u64(), Some(0));
    assert_eq!(
        receipt["counts"]["assessment_history_items"].as_u64(),
        Some(0)
    );
    assert_eq!(receipt["counts"]["assessment_work_items"].as_u64(), Some(2));
    assert!(
        receipt["source_kind_counts"]
            .as_object()
            .is_some_and(|counts| !counts.contains_key("source_kind_assessment"))
    );
    assert_eq!(
        receipt["trace_kind_counts"]["view_model_trace_kind_backend_coverage_item"].as_u64(),
        Some(9)
    );
    assert_eq!(
        receipt["trace_kind_counts"]["view_model_trace_kind_claim_detail_item"].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["trace_source_counts"]["view_model_trace_unique_source_records"].as_u64(),
        Some(5)
    );
    assert_eq!(receipt["view_item_ids"].as_array().map(Vec::len), Some(44));
    assert!(receipt["view_item_ids"].as_array().is_some_and(|ids| {
        ids.iter().any(|id| {
            id.as_str()
                == Some("view_model.claim:belvedere::ingestion::record_identity_is_preserved")
        })
    }));
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(5)
    );
}

fn assert_view_model_receipt_identity(receipt: &Value) {
    assert!(
        receipt["receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.view_model:"))
    );
    assert_eq!(
        receipt["receipt_id"].as_str().map(str::to_owned),
        receipt["receipt_digest"]
            .as_str()
            .map(|digest| format!("belvedere.receipt.view_model:{digest}"))
    );
}

fn assert_auditor_navigation_receipt_source(receipt: &Value) {
    assert!(
        receipt["source_landscape_receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.landscape:"))
    );
    assert!(is_hex_digest(&receipt["source_landscape_receipt_digest"]));
}

fn assert_landscape_diff_review_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.view_model.landscape_diff_review")
    );
    assert!(
        receipt["receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.landscape_diff_review:"))
    );
    assert_eq!(
        receipt["receipt_id"].as_str().map(str::to_owned),
        receipt["receipt_digest"]
            .as_str()
            .map(|digest| { format!("belvedere.receipt.landscape_diff_review:{digest}") })
    );
    assert_eq!(receipt["counts"]["total_items"].as_u64(), Some(6));
    assert_eq!(receipt["counts"]["blocking_items"].as_u64(), Some(1));
    assert_eq!(receipt["counts"]["review_items"].as_u64(), Some(4));
    assert_eq!(receipt["counts"]["informational_items"].as_u64(), Some(1));
    assert_eq!(
        receipt["change_kind_counts"]["diff_review_change_kind_added"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["review_priority_counts"]["diff_review_priority_blocking"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["review_action_counts"]["diff_review_action_triage_new_gap"].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["trace_source_counts"]["diff_review_trace_unique_source_records"].as_u64(),
        Some(3)
    );
    assert!(
        receipt["source_diff_receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.landscape_diff:"))
    );
    assert!(is_hex_digest(&receipt["source_diff_receipt_digest"]));
    assert_eq!(receipt["review_item_ids"].as_array().map(Vec::len), Some(6));
    assert_eq!(
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(7)
    );
}

fn assert_ui_ir_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_ui_ir_receipt_identity(receipt);
    assert_ui_ir_receipt_source_with_prefix(receipt, "belvedere.receipt.view_model:");
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(14));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_HISTORY].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_QUEUE].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_WORK].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_CLAIM_DETAIL].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["trace_source_counts"]["trace_unique_source_records"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["source_locator_counts"]["ui_ir_source_navigation_items"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["source_locator_counts"]["ui_ir_source_locator_items"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["source_locator_counts"]["ui_ir_source_locator_widgets"].as_u64(),
        Some(7)
    );
    assert_eq!(receipt["output_ids"].as_array().map(Vec::len), Some(159));
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_minted_standard"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_section"].as_u64(),
        Some(14)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_viewport"].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_section_root_widget"].as_u64(),
        Some(14)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_trace_widget"].as_u64(),
        Some(62)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_trace_widget_ref"].as_u64(),
        Some(62)
    );
    assert!(
        receipt["trace_widget_digests"]
            .as_array()
            .is_some_and(|digests| { digests.len() == 62 && digests.iter().all(is_hex_digest) })
    );
    assert!(
        receipt["section_ids"]
            .as_array()
            .is_some_and(|section_ids| {
                section_ids
                    .iter()
                    .any(|section_id| section_id.as_str() == Some(UI_IR_SECTION_REVIEW_WORKFLOW))
            })
    );
    assert!(
        receipt["section_ids"]
            .as_array()
            .is_some_and(|section_ids| {
                section_ids
                    .iter()
                    .any(|section_id| section_id.as_str() == Some(UI_IR_SECTION_CLAIM_DETAIL))
            })
    );
    assert!(
        receipt["section_ids"]
            .as_array()
            .is_some_and(|section_ids| {
                section_ids
                    .iter()
                    .any(|section_id| section_id.as_str() == Some(UI_IR_SECTION_ASSESSMENT_HISTORY))
            })
    );
    assert!(
        receipt["section_ids"]
            .as_array()
            .is_some_and(|section_ids| {
                section_ids
                    .iter()
                    .any(|section_id| section_id.as_str() == Some(UI_IR_SECTION_ASSESSMENT_WORK))
            })
    );
}

fn assert_landscape_diff_review_ui_ir_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_diff_review_projection")
    );
    assert_ui_ir_receipt_identity(receipt);
    assert_ui_ir_receipt_source_with_prefix(receipt, "belvedere.receipt.landscape_diff_review:");
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(3));
    assert_eq!(receipt["counts"]["widget_count"].as_u64(), Some(14));
    assert_eq!(receipt["counts"]["trace_ref_count"].as_u64(), Some(22));
    assert_eq!(
        receipt["section_widget_counts"][DIFF_REVIEW_UI_IR_SECTION_SUMMARY].as_u64(),
        Some(1)
    );
    assert_eq!(
        receipt["section_widget_counts"][DIFF_REVIEW_UI_IR_SECTION_ITEMS].as_u64(),
        Some(6)
    );
    assert_eq!(
        receipt["section_widget_counts"][DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["source_locator_counts"]["ui_ir_source_navigation_items"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["source_locator_counts"]["ui_ir_source_locator_items"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["source_locator_counts"]["ui_ir_source_locator_widgets"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["trace_source_counts"]["trace_unique_source_records"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["trace_source_counts"]["trace_source_links"].as_u64(),
        Some(22)
    );
    assert_eq!(receipt["output_ids"].as_array().map(Vec::len), Some(41));
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_minted_standard"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_section"].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_viewport"].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_section_root_widget"].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_trace_widget"].as_u64(),
        Some(14)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_trace_widget_ref"].as_u64(),
        Some(14)
    );
    assert!(
        receipt["section_ids"]
            .as_array()
            .is_some_and(|section_ids| {
                section_ids
                    .iter()
                    .any(|section_id| section_id.as_str() == Some(DIFF_REVIEW_UI_IR_SECTION_ITEMS))
            })
    );
}

fn assert_ui_ir_receipt_source_with_prefix(receipt: &Value, receipt_id_prefix: &str) {
    assert!(
        receipt["source_view_model_receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with(receipt_id_prefix))
    );
    assert!(is_hex_digest(&receipt["source_view_model_receipt_digest"]));
}

fn assert_ui_ir_receipt_identity(receipt: &Value) {
    assert!(
        receipt["receipt_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.receipt.ui_ir:"))
    );
    assert_eq!(
        receipt["receipt_id"].as_str().map(str::to_owned),
        receipt["receipt_digest"]
            .as_str()
            .map(|digest| format!("belvedere.receipt.ui_ir:{digest}"))
    );
}

fn assert_previous_ui_ir_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_ui_ir_receipt_identity(receipt);
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(14));
    assert_eq!(receipt["counts"]["widget_count"].as_u64(), Some(44));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_CLAIM_DETAIL].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_HISTORY].as_u64(),
        Some(0)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_WORK].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["trace_source_counts"]["trace_unique_source_records"].as_u64(),
        Some(5)
    );
    assert!(
        receipt["section_ids"]
            .as_array()
            .is_some_and(|section_ids| section_ids
                .iter()
                .any(|section_id| section_id.as_str() == Some(UI_IR_SECTION_REVIEW_WORKFLOW)))
    );
}

fn assert_auditor_navigation_model(model: &Value) {
    assert_eq!(model["overview"]["claim_count"].as_u64(), Some(4));
    assert_eq!(model["overview"]["root_count"].as_u64(), Some(3));
    assert_eq!(
        model["overview"]["assessment_recency"]["current_count"].as_u64(),
        Some(2)
    );
    assert_eq!(model["filter_facets"].as_array().map(Vec::len), Some(12));
    assert_eq!(model["backend_coverage"].as_array().map(Vec::len), Some(12));
    assert_eq!(model["claim_explorer"].as_array().map(Vec::len), Some(4));
    assert_eq!(model["proof_chains"].as_array().map(Vec::len), Some(4));
    assert_eq!(model["review_workflow"].as_array().map(Vec::len), Some(4));
    assert!(
        model["review_workflow"]
            .as_array()
            .is_some_and(|tasks| { tasks.iter().all(|task| is_hex_digest(&task["task_digest"])) })
    );
    assert_eq!(
        model["assessment_history"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(model["source_record_ids"].as_array().map(Vec::len), Some(7));
    assert!(model["source_navigation"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["source_record_id"].as_str()
                == Some("proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani")
                && item["source_locator"].as_str()
                    == Some(
                        "registry_dump.proof_records[evidence=belvedere::ingestion::DecodedRegistrySnapshot, verifier=kani]",
                    )
        })
    }));
    assert!(model["claim_explorer"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
        })
    }));
}

fn assert_auditor_overview(overview: &Value) {
    assert_eq!(overview["claim_count"].as_u64(), Some(4));
    assert_eq!(overview["root_count"].as_u64(), Some(3));
    assert_eq!(
        overview["assessment_recency"]["current_count"].as_u64(),
        Some(2)
    );
}

fn assert_backend_coverage(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(12));
    assert!(items.as_array().is_some_and(|coverage| {
        coverage.iter().all(|item| {
            item["claim_id"]
                .as_str()
                .is_some_and(|claim_id| !claim_id.is_empty())
                && item["verifier"]
                    .as_str()
                    .is_some_and(|verifier| !verifier.is_empty())
                && item["covered"].as_bool().is_some()
                && item["trace"]["source_record_ids"].as_array().is_some()
        })
    }));
    assert!(items.as_array().is_some_and(|coverage| {
        coverage.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
                && item["verifier"].as_str() == Some("kani")
                && item["covered"].as_bool() == Some(true)
        })
    }));
}

fn assert_previous_backend_coverage(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(9));
    assert!(items.as_array().is_some_and(|coverage| {
        coverage.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["verifier"].as_str() == Some("kani")
                && item["covered"].as_bool() == Some(true)
        })
    }));
}

fn assert_claim_explorer(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(4));
    assert!(items.as_array().is_some_and(|claims| {
        claims.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
                && item["root"].as_bool() == Some(false)
                && item["verifiers"]
                    .as_array()
                    .is_some_and(|verifiers| verifiers.iter().any(|verifier| verifier == "kani"))
                && item["gap_count"].as_u64() == Some(2)
                && item["filter_ids"].as_array().is_some_and(|filters| {
                    filters
                        .iter()
                        .any(|filter| filter.as_str() == Some("belvedere.filter.priority:high"))
                })
        })
    }));
}

fn assert_previous_claim_explorer(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(3));
    assert!(items.as_array().is_some_and(|claims| {
        claims.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["verifiers"]
                    .as_array()
                    .is_some_and(|verifiers| verifiers.iter().any(|verifier| verifier == "kani"))
        })
    }));
}

fn assert_filter_facets(facets: &Value) {
    assert_eq!(facets.as_array().map(Vec::len), Some(12));
    assert!(facets.as_array().is_some_and(|items| {
        items.iter().all(|facet| {
            facet["facet_id"]
                .as_str()
                .is_some_and(|facet_id| !facet_id.is_empty())
                && facet["group"]
                    .as_str()
                    .is_some_and(|group| !group.is_empty())
                && facet["value"]
                    .as_str()
                    .is_some_and(|value| !value.is_empty())
                && facet["label"]
                    .as_str()
                    .is_some_and(|label| !label.is_empty())
                && facet["trace"]["claim_id"]
                    .as_str()
                    .is_some_and(|claim_id| !claim_id.is_empty())
        })
    }));
    assert!(facets.as_array().is_some_and(|items| {
        items.iter().any(|facet| {
            facet["facet_id"].as_str() == Some("belvedere.filter.verifier:kani")
                && facet["group"].as_str() == Some("verifier")
                && facet["value"].as_str() == Some("kani")
                && facet["count"].as_u64() == Some(2)
                && facet["trace"]["claim_id"].as_str() == Some("belvedere.filter.verifier:kani")
        })
    }));
    assert!(facets.as_array().is_some_and(|items| {
        items.iter().any(|facet| {
            facet["facet_id"].as_str() == Some("belvedere.filter.priority:high")
                && facet["group"].as_str() == Some("priority")
                && facet["value"].as_str() == Some("high")
                && facet["count"].as_u64() == Some(3)
        })
    }));
    assert!(facets.as_array().is_some_and(|items| {
        items.iter().any(|facet| {
            facet["facet_id"].as_str()
                == Some(
                    "belvedere.filter.assessment_queue_group:assessment_queue.strengthening:assertion_strength+adversarial_coverage",
                )
                && facet["group"].as_str() == Some("assessment_queue_group")
                && facet["value"].as_str()
                    == Some(
                        "assessment_queue.strengthening:assertion_strength+adversarial_coverage",
                    )
                && facet["label"].as_str()
                    == Some(
                        "Assessment queue group strengthening / assertion_strength, adversarial_coverage",
                    )
                && facet["count"].as_u64() == Some(1)
        })
    }));
}

fn assert_source_navigation_map(map: &Value) {
    assert_eq!(map.as_array().map(Vec::len), Some(7));
    assert!(map.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["source_record_id"]
                .as_str()
                .is_some_and(|source_record_id| !source_record_id.is_empty())
                && item["source_locator"]
                    .as_str()
                    .is_some_and(|source_locator| !source_locator.is_empty())
                && item["label"]
                    .as_str()
                    .is_some_and(|label| !label.is_empty())
                && item["trace"]["source_record_ids"]
                    .as_array()
                    .is_some_and(|source_record_ids| !source_record_ids.is_empty())
        })
    }));
    assert!(map.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["source_record_id"].as_str()
                == Some("proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani")
                && item["source_kind"].as_str() == Some("proof_record")
                && item["source_locator"].as_str()
                    == Some(
                        "registry_dump.proof_records[evidence=belvedere::ingestion::DecodedRegistrySnapshot, verifier=kani]",
                    )
        })
    }));
    assert!(map.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["source_record_id"]
                .as_str()
                .is_some_and(|source_record_id| source_record_id.starts_with("assessment:"))
                && item["artifact_kind"].as_str() == Some("proof_assessments")
                && item["source_kind"].as_str() == Some("assessment")
        })
    }));
}

fn assert_claim_details(details: &Value) {
    assert_eq!(details.as_array().map(Vec::len), Some(4));
    assert!(details.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["claim_id"]
                .as_str()
                .is_some_and(|claim_id| !claim_id.is_empty())
                && item["root"].as_bool().is_some()
                && item["basis_ids"].as_array().is_some()
                && item["root_ids"].as_array().is_some()
                && item["contract_fragments"].as_array().is_some()
                && item["proof_statuses"].as_array().is_some()
                && item["assessment_ids"].as_array().is_some()
                && item["gap_summaries"].as_array().is_some()
                && item["review_state"]
                    .as_str()
                    .is_some_and(|review_state| !review_state.is_empty())
                && item["next_action"]
                    .as_str()
                    .is_some_and(|next_action| !next_action.is_empty())
                && item["source_record_count"].as_u64().is_some()
                && item["trace"]["source_record_ids"]
                    .as_array()
                    .is_some_and(|source_record_ids| !source_record_ids.is_empty())
        })
    }));
    assert!(details.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
                && item["root"].as_bool() == Some(false)
                && item["basis_ids"].as_array().is_some_and(|basis_ids| {
                    basis_ids.iter().any(|basis_id| {
                        basis_id.as_str() == Some("belvedere::ingestion::RawAmenableArtifact")
                    })
                })
                && item["contract_fragments"]
                    .as_array()
                    .is_some_and(|fragments| {
                        fragments.iter().any(|fragment| {
                            fragment
                                .as_str()
                                .is_some_and(|text| text.contains("kani ensures"))
                        })
                    })
                && item["review_state"].as_str() == Some("elevated")
                && item["next_action"].as_str() == Some("review trusted or opaque witness support")
        })
    }));
    assert!(details.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["proof_statuses"].as_array().is_some_and(|statuses| {
                    statuses.iter().any(|status| {
                        status.as_str().is_some_and(|text| {
                            text.contains("status verified")
                                && text.contains("scope bounded harness")
                        })
                    })
                })
        })
    }));
}

fn assert_proof_chains(chains: &Value) {
    assert_eq!(chains.as_array().map(Vec::len), Some(4));
    assert!(chains.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["claim_id"]
                .as_str()
                .is_some_and(|claim_id| !claim_id.is_empty())
                && item["root_ids"].as_array().is_some()
                && item["backend_coverage"]
                    .as_array()
                    .is_some_and(|coverage| coverage.len() == 3)
                && item["topology"]["transitive_claim_count"]
                    .as_u64()
                    .is_some()
                && item["trace"]["source_record_ids"]
                    .as_array()
                    .is_some_and(|source_record_ids| !source_record_ids.is_empty())
        })
    }));
    assert!(chains.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
                && item["root_ids"].as_array().is_some_and(|root_ids| {
                    root_ids.iter().any(|root_id| {
                        root_id.as_str() == Some("belvedere::ingestion::RawAmenableArtifact")
                    })
                })
                && item["backend_coverage"].as_array().is_some_and(|coverage| {
                    coverage.iter().any(|backend| {
                        backend["verifier"].as_str() == Some("kani")
                            && backend["covered_claim_count"]
                                .as_u64()
                                .is_some_and(|count| count > 0)
                    })
                })
        })
    }));
    assert!(chains.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["review_state"].as_str() == Some("blocked")
                && item["gap_count"]
                    .as_u64()
                    .is_some_and(|gap_count| gap_count > 0)
                && item["next_action"]
                    .as_str()
                    .is_some_and(|next_action| !next_action.is_empty())
        })
    }));
}

fn assert_review_workflow_queue(queue: &Value) {
    assert_eq!(queue.as_array().map(Vec::len), Some(4));
    assert!(queue.as_array().is_some_and(|tasks| {
        tasks.iter().all(|task| {
            is_hex_digest(&task["task_digest"])
                && task["task_id"].as_str().is_some_and(|id| !id.is_empty())
        })
    }));
    assert!(queue.as_array().is_some_and(|tasks| {
        tasks.iter().any(|task| {
            task["priority"].as_str() == Some("blocking")
                && task["lane"]
                    .as_str()
                    .is_some_and(|lane| lane.starts_with("blocked:"))
                && task["action"]
                    .as_str()
                    .is_some_and(|action| !action.is_empty())
        })
    }));
}

fn assert_previous_review_workflow_queue(queue: &Value) {
    assert_eq!(queue.as_array().map(Vec::len), Some(3));
    assert!(queue.as_array().is_some_and(|tasks| {
        tasks.iter().all(|task| {
            is_hex_digest(&task["task_digest"])
                && task["task_id"].as_str().is_some_and(|id| !id.is_empty())
        })
    }));
}

fn assert_assessment_history(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(2));
    assert!(items.as_array().is_some_and(|history| {
        history.iter().any(|item| {
            item["proof_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["current_assessment_id"].as_str() == Some("assess-belvedere-ingestion-1")
                && item["history_count"].as_u64() == Some(1)
                && item["recommendation_flow"].as_array().is_some_and(|flow| {
                    flow.iter()
                        .any(|entry| entry.as_str() == Some("Strengthen"))
                })
        })
    }));
}

fn assert_empty_assessment_history(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(0));
}

fn assert_assessment_queue(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(2));
    assert!(items.as_array().is_some_and(|queue| {
        queue.iter().any(|item| {
            item["assessment_id"].as_str() == Some("assess-belvedere-ingestion-1")
                && item["proof_id"].as_str()
                    == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["current"].as_bool() == Some(true)
                && item["queue_lane"].as_str() == Some("strengthening")
                && item["queue_group_id"].as_str()
                    == Some(
                        "assessment_queue.strengthening:assertion_strength+adversarial_coverage",
                    )
                && item["queue_group_label"].as_str()
                    == Some("strengthening / assertion_strength, adversarial_coverage")
        })
    }));
}

fn assert_empty_assessment_queue(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(0));
}

fn assert_assessment_work(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(2));
    assert!(items.as_array().is_some_and(|work| {
        work.iter().any(|item| {
            item["work_id"].as_str()
                == Some(
                    "assessment_work.missing_assessment:view_model.gap:belvedere::ingestion::DecodedRegistrySnapshot:missing_assessment",
                )
                &&
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
                && item["queue_lane"].as_str() == Some("assessment")
                && item["action"].as_str() == Some("assess proof coverage")
                && item["resolution_label"].as_str() == Some("collect proof assessment")
        })
    }));
}

fn assert_witness_surface(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(1));
    assert!(items.as_array().is_some_and(|witnesses| {
        witnesses.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
                && item["verifier"].as_str() == Some("kani")
                && item["support_kind"].as_str() == Some("checked_with_trusted")
                && item["checked"].as_u64() == Some(1)
                && item["trusted"].as_u64() == Some(1)
        })
    }));
}

fn assert_provenance_roots(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(3));
    assert!(items.as_array().is_some_and(|roots| {
        roots.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::RawAmenableArtifact")
                && item["root_kind"].as_str() == Some("unmodeled_root")
                && item["next_action"].as_str() == Some("establish modeled proof coverage")
                && item["gap_count"].as_u64() == Some(1)
                && item["source_record_ids"]
                    .as_array()
                    .is_some_and(|source_record_ids| {
                        source_record_ids.iter().any(|source_record_id| {
                            source_record_id.as_str()
                                == Some(
                                    "evidence_link:belvedere::ingestion::DecodedRegistrySnapshot#0",
                                )
                        })
                    })
        })
    }));
}

fn assert_previous_provenance_roots(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(2));
    assert!(items.as_array().is_some_and(|roots| {
        roots.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["root_kind"].as_str() == Some("proof_registration_root")
        })
    }));
}

fn assert_gap_items(items: &Value) {
    assert_eq!(items.as_array().map(Vec::len), Some(4));
    assert!(items.as_array().is_some_and(|gaps| {
        gaps.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
                && item["kind"].as_str() == Some("missing_assessment")
                && item["severity"].as_str() == Some("review")
                && item["queue_lane"].as_str() == Some("assessment")
        })
    }));
}

fn assert_previous_gap_items(items: &Value) {
    assert!(items.as_array().is_some_and(|gaps| {
        !gaps.is_empty()
            && gaps.iter().any(|item| {
                item["kind"].as_str() == Some("missing_proof")
                    && item["severity"].as_str() == Some("blocking")
            })
    }));
}

fn assert_previous_source_navigation_map(map: &Value) {
    assert_eq!(map.as_array().map(Vec::len), Some(5));
    assert!(map.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["source_record_id"]
                .as_str()
                .is_some_and(|source_record_id| !source_record_id.is_empty())
                && item["source_locator"]
                    .as_str()
                    .is_some_and(|source_locator| !source_locator.is_empty())
                && item["trace"]["source_record_ids"]
                    .as_array()
                    .is_some_and(|source_record_ids| !source_record_ids.is_empty())
        })
    }));
    assert!(map.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["source_record_id"].as_str()
                == Some("kani_proof:belvedere::ingestion::record_identity_is_preserved")
                && item["source_kind"].as_str() == Some("kani_proof")
                && item["source_locator"].as_str()
                    == Some(
                        "registry_dump.kani_proofs[id=belvedere::ingestion::record_identity_is_preserved]",
                    )
        })
    }));
}

fn assert_previous_proof_chains(chains: &Value) {
    assert_eq!(chains.as_array().map(Vec::len), Some(3));
    assert!(chains.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["claim_id"]
                .as_str()
                .is_some_and(|claim_id| !claim_id.is_empty())
                && item["root_ids"].as_array().is_some()
                && item["backend_coverage"]
                    .as_array()
                    .is_some_and(|coverage| coverage.len() == 3)
                && item["topology"]["max_depth"].as_u64().is_some()
        })
    }));
    assert!(chains.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["verifiers"].as_array().is_some_and(|verifiers| {
                    verifiers
                        .iter()
                        .any(|verifier| verifier.as_str() == Some("kani"))
                })
        })
    }));
}

fn assert_previous_claim_details(details: &Value) {
    assert_eq!(details.as_array().map(Vec::len), Some(3));
    assert!(details.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["claim_id"]
                .as_str()
                .is_some_and(|claim_id| !claim_id.is_empty())
                && item["root"].as_bool().is_some()
                && item["basis_ids"].as_array().is_some()
                && item["root_ids"].as_array().is_some()
                && item["review_state"]
                    .as_str()
                    .is_some_and(|review_state| !review_state.is_empty())
                && item["next_action"]
                    .as_str()
                    .is_some_and(|next_action| !next_action.is_empty())
                && item["trace"]["source_record_ids"]
                    .as_array()
                    .is_some_and(|source_record_ids| !source_record_ids.is_empty())
        })
    }));
    assert!(details.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::RawAmenableArtifact")
                && item["root"].as_bool() == Some(true)
                && item["review_state"].as_str() == Some("blocked")
        })
    }));
    assert!(details.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
                && item["verifiers"].as_array().is_some_and(|verifiers| {
                    verifiers
                        .iter()
                        .any(|verifier| verifier.as_str() == Some("kani"))
                })
        })
    }));
}

fn assert_previous_filter_facets(facets: &Value) {
    assert_eq!(facets.as_array().map(Vec::len), Some(8));
    assert!(facets.as_array().is_some_and(|items| {
        items.iter().any(|facet| {
            facet["facet_id"].as_str() == Some("belvedere.filter.claim_family:belvedere::ingestion")
                && facet["group"].as_str() == Some("claim_family")
                && facet["count"].as_u64() == Some(3)
        })
    }));
    assert!(facets.as_array().is_some_and(|items| {
        items.iter().any(|facet| {
            facet["facet_id"].as_str() == Some("belvedere.filter.verifier:kani")
                && facet["group"].as_str() == Some("verifier")
                && facet["count"].as_u64() == Some(2)
        })
    }));
}

fn assert_previous_auditor_navigation_model(model: &Value) {
    assert_eq!(model["overview"]["claim_count"].as_u64(), Some(3));
    assert_eq!(model["overview"]["root_count"].as_u64(), Some(2));
    assert_eq!(
        model["overview"]["assessment_recency"]["current_count"].as_u64(),
        Some(0)
    );
    assert_eq!(model["filter_facets"].as_array().map(Vec::len), Some(8));
    assert_eq!(model["backend_coverage"].as_array().map(Vec::len), Some(9));
    assert_eq!(model["claim_explorer"].as_array().map(Vec::len), Some(3));
    assert_eq!(model["proof_chains"].as_array().map(Vec::len), Some(3));
    assert_eq!(model["review_workflow"].as_array().map(Vec::len), Some(3));
    assert_eq!(
        model["assessment_history"].as_array().map(Vec::len),
        Some(0)
    );
    assert_eq!(model["assessment_queue"].as_array().map(Vec::len), Some(0));
    assert_eq!(model["assessment_work"].as_array().map(Vec::len), Some(2));
    assert_eq!(model["source_record_ids"].as_array().map(Vec::len), Some(5));
    assert!(model["claim_explorer"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::record_identity_is_preserved")
        })
    }));
}

fn assert_previous_auditor_overview(overview: &Value) {
    assert_eq!(overview["claim_count"].as_u64(), Some(3));
    assert_eq!(overview["root_count"].as_u64(), Some(2));
    assert_eq!(
        overview["assessment_recency"]["current_count"].as_u64(),
        Some(0)
    );
}

fn assert_landscape_diff_review_model(model: &Value) {
    assert_eq!(model["summary"]["added_items"].as_u64(), Some(4));
    assert_eq!(model["summary"]["removed_items"].as_u64(), Some(1));
    assert_eq!(model["summary"]["changed_items"].as_u64(), Some(1));
    assert_eq!(model["summary"]["blocking_items"].as_u64(), Some(1));
    assert_eq!(model["summary"]["review_items"].as_u64(), Some(4));
    assert_eq!(model["summary"]["informational_items"].as_u64(), Some(1));
    assert_eq!(model["items"].as_array().map(Vec::len), Some(6));
    assert_eq!(model["source_record_ids"].as_array().map(Vec::len), Some(7));
    assert!(model["items"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["review_priority"].as_str() == Some("blocking")
                && item["review_action"].as_str() == Some("triage_new_gap")
                && item["subject_kind"].as_str() == Some("gap")
        })
    }));
    assert!(model["items"].as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["item_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("belvedere.diff_review.item:"))
                && item["diff_item_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("belvedere.diff."))
        })
    }));
}

fn assert_landscape_diff_review_workflow_queue(queue: &Value) {
    assert_eq!(queue.as_array().map(Vec::len), Some(6));
    assert!(queue.as_array().is_some_and(|items| {
        items.iter().all(|item| {
            item["item_id"]
                .as_str()
                .is_some_and(|id| id.starts_with("belvedere.diff_review.item:"))
                && item["diff_item_id"]
                    .as_str()
                    .is_some_and(|id| id.starts_with("belvedere.diff."))
        })
    }));
    assert!(queue.as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["review_priority"].as_str() == Some("blocking")
                && item["review_action"].as_str() == Some("triage_new_gap")
                && item["subject_kind"].as_str() == Some("gap")
        })
    }));
}

fn assert_ui_ir_projection(projection: &Value) {
    assert!(projection.get("tree").is_none());
    assert!(projection.get("proofs").is_none());
    assert_eq!(projection["sections"].as_array().map(Vec::len), Some(14));
    assert_eq!(
        projection["trace_widgets"].as_array().map(Vec::len),
        Some(62)
    );
    assert!(
        projection["trace_widgets"]
            .as_array()
            .is_some_and(|widgets| widgets
                .iter()
                .all(|widget| is_hex_digest(&widget["trace_widget_digest"])))
    );
    assert!(projection["trace_widgets"].as_array().is_some_and(|widgets| {
        widgets.iter().any(|widget| {
            widget["label"].as_str().is_some_and(|label| {
                label.contains(
                    "locator registry_dump.proof_records[evidence=belvedere::ingestion::DecodedRegistrySnapshot, verifier=kani]",
                )
            })
        })
    }));
    assert_eq!(
        projection["viewport_verifications"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        projection["minted_standards"].as_array().map(Vec::len),
        Some(4)
    );
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_REVIEW_WORKFLOW))
    }));
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_CLAIM_DETAIL))
    }));
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_ASSESSMENT_HISTORY))
    }));
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_ASSESSMENT_WORK))
    }));
    assert!(
        projection["minted_standards"]
            .as_array()
            .is_some_and(|standards| standards
                .iter()
                .any(|standard| standard["prop"].as_str() == Some("elicit_ui::WcagVerified")))
    );
}

fn assert_ui_ir_sections(sections: &Value) {
    assert_eq!(sections.as_array().map(Vec::len), Some(14));
    assert!(sections.as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_REVIEW_WORKFLOW))
    }));
    assert!(sections.as_array().is_some_and(|sections| {
        sections.iter().all(|section| {
            section["name"]
                .as_str()
                .is_some_and(|name| !name.is_empty())
                && section["root_widget_id"].as_u64().is_some()
                && section["widget_count"].as_u64().is_some()
        })
    }));
}

fn assert_ui_ir_trace_widgets(widgets: &Value) {
    assert_eq!(widgets.as_array().map(Vec::len), Some(62));
    assert!(widgets.as_array().is_some_and(|widgets| {
        widgets
            .iter()
            .all(|widget| is_hex_digest(&widget["trace_widget_digest"]))
    }));
    assert!(widgets.as_array().is_some_and(|widgets| {
        widgets.iter().any(|widget| {
            widget["section_id"].as_str() == Some(UI_IR_SECTION_SOURCE_NAVIGATION)
                && widget["source_record_ids"]
                    .as_array()
                    .is_some_and(|ids| !ids.is_empty())
        })
    }));
}

fn assert_previous_ui_ir_trace_widgets(widgets: &Value) {
    assert_eq!(widgets.as_array().map(Vec::len), Some(44));
    assert!(widgets.as_array().is_some_and(|widgets| {
        widgets
            .iter()
            .all(|widget| is_hex_digest(&widget["trace_widget_digest"]))
    }));
    assert!(widgets.as_array().is_some_and(|widgets| {
        widgets.iter().any(|widget| {
            widget["section_id"].as_str() == Some(UI_IR_SECTION_SOURCE_NAVIGATION)
                && widget["source_record_ids"]
                    .as_array()
                    .is_some_and(|ids| !ids.is_empty())
        })
    }));
}

fn assert_ui_ir_viewports(viewports: &Value) {
    assert_eq!(viewports.as_array().map(Vec::len), Some(3));
    assert!(viewports.as_array().is_some_and(|viewports| {
        viewports.iter().any(|viewport| {
            viewport["width"].as_u64() == Some(1440)
                && viewport["height"].as_u64() == Some(960)
                && viewport["node_count"]
                    .as_u64()
                    .is_some_and(|count| count > 0)
        })
    }));
}

fn assert_ui_ir_minted_standards(standards: &Value) {
    assert_eq!(standards.as_array().map(Vec::len), Some(4));
    assert!(standards.as_array().is_some_and(|standards| {
        standards
            .iter()
            .any(|standard| standard["prop"].as_str() == Some("elicit_ui::WcagVerified"))
    }));
    assert!(standards.as_array().is_some_and(|standards| {
        standards.iter().all(|standard| {
            standard["standard"]
                .as_str()
                .is_some_and(|name| name.contains("ElicitUi"))
                && standard["purpose"]
                    .as_str()
                    .is_some_and(|purpose| !purpose.is_empty())
        })
    }));
}

fn assert_landscape_diff_review_ui_ir_projection(projection: &Value) {
    assert!(projection.get("tree").is_none());
    assert!(projection.get("proofs").is_none());
    assert_eq!(projection["sections"].as_array().map(Vec::len), Some(3));
    assert_eq!(
        projection["trace_widgets"].as_array().map(Vec::len),
        Some(14)
    );
    assert!(
        projection["trace_widgets"]
            .as_array()
            .is_some_and(|widgets| widgets
                .iter()
                .all(|widget| is_hex_digest(&widget["trace_widget_digest"])))
    );
    assert!(
        projection["trace_widgets"]
            .as_array()
            .is_some_and(|widgets| {
                widgets.iter().any(|widget| {
                    widget["section_id"].as_str() == Some(DIFF_REVIEW_UI_IR_SECTION_ITEMS)
                        && widget["label"]
                            .as_str()
                            .is_some_and(|label| label.contains("action triage new gap"))
                })
            })
    );
    assert_eq!(
        projection["viewport_verifications"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        projection["minted_standards"].as_array().map(Vec::len),
        Some(4)
    );
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(DIFF_REVIEW_UI_IR_SECTION_SUMMARY))
    }));
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(DIFF_REVIEW_UI_IR_SECTION_ITEMS))
    }));
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections.iter().any(|section| {
            section["id"].as_str() == Some(DIFF_REVIEW_UI_IR_SECTION_SOURCE_NAVIGATION)
        })
    }));
}

fn assert_landscape_diff_review_ui_ir_sections(sections: &Value) {
    assert_eq!(sections.as_array().map(Vec::len), Some(3));
    assert!(sections.as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(DIFF_REVIEW_UI_IR_SECTION_SUMMARY))
    }));
    assert!(sections.as_array().is_some_and(|sections| {
        sections.iter().all(|section| {
            section["name"]
                .as_str()
                .is_some_and(|name| !name.is_empty())
                && section["root_widget_id"].as_u64().is_some()
                && section["widget_count"].as_u64().is_some()
        })
    }));
}

fn assert_landscape_diff_review_ui_ir_trace_widgets(widgets: &Value) {
    assert_eq!(widgets.as_array().map(Vec::len), Some(14));
    assert!(widgets.as_array().is_some_and(|widgets| {
        widgets
            .iter()
            .all(|widget| is_hex_digest(&widget["trace_widget_digest"]))
    }));
    assert!(widgets.as_array().is_some_and(|widgets| {
        widgets.iter().any(|widget| {
            widget["section_id"].as_str() == Some(DIFF_REVIEW_UI_IR_SECTION_ITEMS)
                && widget["label"]
                    .as_str()
                    .is_some_and(|label| label.contains("action triage new gap"))
        })
    }));
}

fn assert_previous_ui_ir_projection(projection: &Value) {
    assert!(projection.get("tree").is_none());
    assert!(projection.get("proofs").is_none());
    assert_eq!(projection["sections"].as_array().map(Vec::len), Some(14));
    assert_eq!(
        projection["trace_widgets"].as_array().map(Vec::len),
        Some(44)
    );
    assert!(
        projection["trace_widgets"]
            .as_array()
            .is_some_and(|widgets| widgets
                .iter()
                .all(|widget| is_hex_digest(&widget["trace_widget_digest"])))
    );
    assert_eq!(
        projection["viewport_verifications"]
            .as_array()
            .map(Vec::len),
        Some(3)
    );
    assert_eq!(
        projection["minted_standards"].as_array().map(Vec::len),
        Some(4)
    );
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_REVIEW_WORKFLOW))
    }));
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_CLAIM_DETAIL))
    }));
    assert!(
        projection["minted_standards"]
            .as_array()
            .is_some_and(|standards| standards
                .iter()
                .any(|standard| standard["prop"].as_str() == Some("elicit_ui::WcagVerified")))
    );
}

fn assert_run_manifest(manifest: &Value) {
    assert_eq!(
        manifest["schema_id"].as_str(),
        Some(BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID)
    );
    assert_eq!(
        manifest["schema_version"].as_u64(),
        Some(u64::from(BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION))
    );
    assert!(
        manifest["run_certificate_id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.run:"))
    );
    assert!(
        manifest["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.manifest.artifacts:"))
    );
    assert_eq!(
        manifest["id"].as_str().map(str::to_owned),
        manifest["digest"]
            .as_str()
            .map(|digest| format!("belvedere.manifest.artifacts:{digest}"))
    );
    assert_eq!(manifest["counts"]["input_artifacts"].as_u64(), Some(2));
    assert_eq!(manifest["counts"]["stage_certificates"].as_u64(), Some(5));
    assert_eq!(manifest["counts"]["output_artifacts"].as_u64(), Some(35));
    assert_eq!(
        manifest["input_role_counts"]["manifest_input_role_registry"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["stage_kind_counts"]["manifest_stage_kind_ui_ir_projection"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_ui_ir_receipt"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_auditor_backend_coverage"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_auditor_overview"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_elicit_ui_trace_widgets"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_stage_certificate"].as_u64(),
        Some(5)
    );
    assert!(
        manifest["invariant_checks"]
            .as_array()
            .is_some_and(|checks| checks.iter().any(|check| {
                check["name"].as_str() == Some("artifact_manifest_output_sources_accounted")
                    && check["passed"].as_bool() == Some(true)
            }))
    );
    assert!(
        manifest["invariant_checks"]
            .as_array()
            .is_some_and(|checks| checks.iter().any(|check| {
                check["name"].as_str() == Some("artifact_manifest_output_evidence_identified")
                    && check["passed"].as_bool() == Some(true)
            }))
    );
    assert_eq!(
        manifest["input_artifacts"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(
        manifest["stage_certificates"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(
        manifest["output_artifacts"].as_array().map(Vec::len),
        Some(35)
    );
    assert!(
        manifest["output_artifacts"]
            .as_array()
            .is_some_and(|artifacts| {
                artifacts.iter().any(|artifact| {
                    artifact["role"].as_str() == Some("ui_ir_projection")
                        && artifact["artifact_kind"].as_str() == Some("elicit_ui_projection")
                        && artifact["artifact_id"]
                            .as_str()
                            .is_some_and(|id| id.starts_with("belvedere.manifest.output_artifact:"))
                        && artifact["artifact_id"].as_str().map(str::to_owned)
                            == artifact["artifact_digest"].as_str().map(|digest| {
                                format!("belvedere.manifest.output_artifact:{digest}")
                            })
                        && artifact["evidence_id"]
                            .as_str()
                            .is_some_and(|id| id.starts_with("belvedere.receipt.ui_ir:"))
                        && is_hex_digest(&artifact["evidence_digest"])
                        && artifact["source_stage_role"].as_str() == Some("ui_ir_projection")
                })
            })
    );
    assert!(manifest_output_artifact_matches(
        manifest,
        "overview",
        "auditor_overview",
        "belvedere.receipt.view_model:",
        "auditor_navigation",
        1,
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "backend_coverage",
        "auditor_backend_coverage",
        "belvedere.receipt.view_model:",
        "auditor_navigation",
        12,
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "ui_ir_trace_widgets",
        "elicit_ui_trace_widgets",
        "belvedere.receipt.ui_ir:",
        "ui_ir_projection",
        62,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "run_certificate",
        "--certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "registry_ingestion_certificate",
        "--registry-ingestion-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "assessment_ingestion_certificate",
        "--assessment-ingestion-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "landscape_modeling_certificate",
        "--landscape-modeling-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "auditor_navigation_certificate",
        "--auditor-navigation-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "ui_ir_projection_certificate",
        "--ui-ir-projection-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "overview",
        "--overview",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "ui_ir_trace_widgets",
        "--ui-ir-trace-widgets",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "ui_ir_outputs",
        "elicit_ui_outputs",
        "belvedere.receipt.ui_ir:",
        "ui_ir_projection",
        159,
    ));
    assert!(manifest_output_artifact_omits_selector(
        manifest,
        "ui_ir_outputs",
    ));
}

fn assert_diff_manifest(manifest: &Value) {
    assert_eq!(
        manifest["schema_id"].as_str(),
        Some(BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID)
    );
    assert!(
        manifest["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.manifest.artifacts:"))
    );
    assert_eq!(
        manifest["id"].as_str().map(str::to_owned),
        manifest["digest"]
            .as_str()
            .map(|digest| format!("belvedere.manifest.artifacts:{digest}"))
    );
    assert_eq!(manifest["counts"]["input_artifacts"].as_u64(), Some(4));
    assert_eq!(manifest["counts"]["stage_certificates"].as_u64(), Some(13));
    assert_eq!(manifest["counts"]["output_artifacts"].as_u64(), Some(99));
    assert_eq!(
        manifest["stage_kind_counts"]["manifest_stage_kind_registry_ingestion"].as_u64(),
        Some(2)
    );
    assert_eq!(
        manifest["stage_kind_counts"]["manifest_stage_kind_landscape_diff"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["stage_kind_counts"]["manifest_stage_kind_landscape_diff_review"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["stage_kind_counts"]["manifest_stage_kind_landscape_diff_review_ui_ir_projection"]
            .as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_landscape_diff_receipt"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_landscape_diff_review_receipt"]
            .as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_elicit_ui_projection"].as_u64(),
        Some(3)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_ui_ir_receipt"].as_u64(),
        Some(3)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_auditor_backend_coverage"].as_u64(),
        Some(3)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_auditor_overview"].as_u64(),
        Some(3)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_elicit_ui_trace_widgets"].as_u64(),
        Some(3)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_stage_certificate"].as_u64(),
        Some(13)
    );
    assert!(
        manifest["invariant_checks"]
            .as_array()
            .is_some_and(|checks| checks
                .iter()
                .all(|check| check["passed"].as_bool() == Some(true)))
    );
    assert_eq!(
        manifest["stage_certificates"].as_array().map(Vec::len),
        Some(13)
    );
    assert!(
        manifest["stage_certificates"]
            .as_array()
            .is_some_and(|certificates| certificates.iter().any(|certificate| {
                certificate["role"].as_str() == Some("landscape_diff")
                    && certificate["stage"].as_str() == Some("landscape_diff")
            }))
    );
    assert!(
        manifest["stage_certificates"]
            .as_array()
            .is_some_and(|certificates| certificates.iter().any(|certificate| {
                certificate["role"].as_str() == Some("landscape_diff_review")
                    && certificate["stage"].as_str() == Some("landscape_diff_review")
            }))
    );
    assert!(
        manifest["stage_certificates"]
            .as_array()
            .is_some_and(|certificates| certificates.iter().any(|certificate| {
                certificate["role"].as_str() == Some("landscape_diff_review_ui_ir_projection")
                    && certificate["stage"].as_str()
                        == Some("landscape_diff_review_ui_ir_projection")
            }))
    );
    assert!(
        manifest["output_artifacts"]
            .as_array()
            .is_some_and(|artifacts| {
                artifacts.iter().any(|artifact| {
                    artifact["role"].as_str() == Some("landscape_diff")
                        && artifact["artifact_kind"].as_str() == Some("landscape_diff")
                        && artifact["artifact_id"]
                            .as_str()
                            .is_some_and(|id| id.starts_with("belvedere.manifest.output_artifact:"))
                        && artifact["artifact_id"].as_str().map(str::to_owned)
                            == artifact["artifact_digest"].as_str().map(|digest| {
                                format!("belvedere.manifest.output_artifact:{digest}")
                            })
                        && artifact["evidence_id"]
                            .as_str()
                            .is_some_and(|id| id.starts_with("belvedere.receipt.landscape_diff:"))
                        && is_hex_digest(&artifact["evidence_digest"])
                        && artifact["item_count"].as_u64() == Some(6)
                })
            })
    );
    assert!(
        manifest["output_artifacts"]
            .as_array()
            .is_some_and(|artifacts| {
                artifacts.iter().any(|artifact| {
                    artifact["role"].as_str() == Some("landscape_diff_review_model")
                        && artifact["artifact_kind"].as_str() == Some("landscape_diff_review_model")
                        && artifact["evidence_id"].as_str().is_some_and(|id| {
                            id.starts_with("belvedere.receipt.landscape_diff_review:")
                        })
                        && artifact["source_stage_role"].as_str() == Some("landscape_diff_review")
                        && artifact["item_count"].as_u64() == Some(6)
                })
            })
    );
    assert!(
        manifest["output_artifacts"]
            .as_array()
            .is_some_and(|artifacts| {
                artifacts.iter().any(|artifact| {
                    artifact["role"].as_str() == Some("landscape_diff_review_ui_ir_projection")
                        && artifact["artifact_kind"].as_str() == Some("elicit_ui_projection")
                        && artifact["evidence_id"]
                            .as_str()
                            .is_some_and(|id| id.starts_with("belvedere.receipt.ui_ir:"))
                        && artifact["source_stage_role"].as_str()
                            == Some("landscape_diff_review_ui_ir_projection")
                        && artifact["item_count"].as_u64() == Some(14)
                })
            })
    );
    assert!(manifest_output_artifact_matches(
        manifest,
        "previous_overview",
        "auditor_overview",
        "belvedere.receipt.view_model:",
        "previous_auditor_navigation",
        1,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "previous_overview",
        "--previous-overview",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "previous_backend_coverage",
        "auditor_backend_coverage",
        "belvedere.receipt.view_model:",
        "previous_auditor_navigation",
        9,
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "current_overview",
        "auditor_overview",
        "belvedere.receipt.view_model:",
        "current_auditor_navigation",
        1,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "current_overview",
        "--current-overview",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "current_backend_coverage",
        "auditor_backend_coverage",
        "belvedere.receipt.view_model:",
        "current_auditor_navigation",
        12,
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "overview",
        "auditor_overview",
        "belvedere.receipt.view_model:",
        "current_auditor_navigation",
        1,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "overview",
        "--overview",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "backend_coverage",
        "auditor_backend_coverage",
        "belvedere.receipt.view_model:",
        "current_auditor_navigation",
        12,
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "previous_ui_ir_trace_widgets",
        "elicit_ui_trace_widgets",
        "belvedere.receipt.ui_ir:",
        "previous_ui_ir_projection",
        44,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "run_certificate",
        "--run-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "diff_certificate",
        "--certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "diff_review_certificate",
        "--diff-review-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "diff_review_ui_ir_certificate",
        "--diff-review-ui-ir-certificate",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "previous_registry_ingestion_certificate",
        "stage_certificate",
        "belvedere.stage.registry_ingestion:",
        "previous_registry_ingestion",
        5,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "previous_registry_ingestion_certificate",
        "--previous-registry-ingestion-certificate",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "previous_assessment_ingestion_certificate",
        "stage_certificate",
        "belvedere.stage.assessment_ingestion:",
        "previous_assessment_ingestion",
        0,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "previous_assessment_ingestion_certificate",
        "--previous-assessment-ingestion-certificate",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "current_ui_ir_projection_certificate",
        "stage_certificate",
        "belvedere.stage.ui_ir_projection:",
        "current_ui_ir_projection",
        159,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "current_ui_ir_projection_certificate",
        "--current-ui-ir-projection-certificate",
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "previous_ui_ir_trace_widgets",
        "--previous-ui-ir-trace-widgets",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "previous_ui_ir_outputs",
        "elicit_ui_outputs",
        "belvedere.receipt.ui_ir:",
        "previous_ui_ir_projection",
        123,
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "current_ui_ir_trace_widgets",
        "elicit_ui_trace_widgets",
        "belvedere.receipt.ui_ir:",
        "current_ui_ir_projection",
        62,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "current_ui_ir_trace_widgets",
        "--current-ui-ir-trace-widgets",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "current_ui_ir_outputs",
        "elicit_ui_outputs",
        "belvedere.receipt.ui_ir:",
        "current_ui_ir_projection",
        159,
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "landscape_diff_review_ui_ir_trace_widgets",
        "elicit_ui_trace_widgets",
        "belvedere.receipt.ui_ir:",
        "landscape_diff_review_ui_ir_projection",
        14,
    ));
    assert!(manifest_output_artifact_has_selector(
        manifest,
        "landscape_diff_review_ui_ir_trace_widgets",
        "--ui-ir-trace-widgets",
    ));
    assert!(manifest_output_artifact_matches(
        manifest,
        "landscape_diff_review_ui_ir_outputs",
        "elicit_ui_outputs",
        "belvedere.receipt.ui_ir:",
        "landscape_diff_review_ui_ir_projection",
        41,
    ));
    assert!(manifest_output_artifact_omits_selector(
        manifest,
        "landscape_diff_review_ui_ir_outputs",
    ));
}

fn manifest_output_artifact_matches(
    manifest: &Value,
    role: &str,
    artifact_kind: &str,
    evidence_prefix: &str,
    source_stage_role: &str,
    item_count: u64,
) -> bool {
    manifest["output_artifacts"]
        .as_array()
        .is_some_and(|artifacts| {
            artifacts.iter().any(|artifact| {
                artifact["role"].as_str() == Some(role)
                    && artifact["artifact_kind"].as_str() == Some(artifact_kind)
                    && artifact["evidence_id"]
                        .as_str()
                        .is_some_and(|id| id.starts_with(evidence_prefix))
                    && is_hex_digest(&artifact["evidence_digest"])
                    && artifact["source_stage_role"].as_str() == Some(source_stage_role)
                    && artifact["item_count"].as_u64() == Some(item_count)
            })
        })
}

fn manifest_output_artifact_has_selector(manifest: &Value, role: &str, selector: &str) -> bool {
    manifest["output_artifacts"]
        .as_array()
        .is_some_and(|artifacts| {
            artifacts.iter().any(|artifact| {
                artifact["role"].as_str() == Some(role)
                    && artifact["cli_selector"].as_str() == Some(selector)
            })
        })
}

fn manifest_output_artifact_omits_selector(manifest: &Value, role: &str) -> bool {
    manifest["output_artifacts"]
        .as_array()
        .is_some_and(|artifacts| {
            artifacts.iter().any(|artifact| {
                artifact["role"].as_str() == Some(role) && artifact.get("cli_selector").is_none()
            })
        })
}

#[test]
fn cli_can_emit_the_run_certificate_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--certificate",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("certificate JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunCertificate)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let certificate: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI certificate output should be parseable JSON")?;
    assert!(
        certificate["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.run:"))
    );
    assert_eq!(
        certificate["schema_id"].as_str(),
        Some(BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID)
    );
    assert_eq!(
        certificate["schema_version"].as_u64(),
        Some(u64::from(BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION))
    );
    assert_eq!(certificate["invocation"]["surface"].as_str(), Some("cli"));
    assert_eq!(
        certificate["invocation"]["command"].as_str(),
        Some(BelvedereInvocationCommand::Run.as_str())
    );
    assert_eq!(
        certificate["invocation"]["output_artifact"].as_str(),
        Some("run_certificate")
    );
    assert_eq!(
        certificate["invocation"]["output_sink"].as_str(),
        Some("stdio")
    );
    assert_eq!(
        certificate["invocation"]["arguments"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert_eq!(
        certificate["stage_certificates"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(
        certificate["stage_certificate_digests"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
    assert_eq!(
        certificate["input_artifact_ids"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(
        certificate["pipeline_claims"].as_array().map(Vec::len),
        Some(8)
    );
    assert_eq!(
        certificate["pipeline_claim_phase_counts"]["phase_ingestion"].as_u64(),
        Some(2)
    );
    assert_eq!(
        certificate["pipeline_claim_phase_counts"]["phase_landscape"].as_u64(),
        Some(2)
    );
    assert_eq!(
        certificate["pipeline_claim_phase_counts"]["phase_view_model"].as_u64(),
        Some(1)
    );
    assert_eq!(
        certificate["pipeline_claim_phase_counts"]["phase_ui_ir"].as_u64(),
        Some(2)
    );
    assert_eq!(
        certificate["pipeline_claim_phase_counts"]["phase_run"].as_u64(),
        Some(1)
    );
    assert!(
        certificate["pipeline_claims"]
            .as_array()
            .is_some_and(
                |claims| claims.iter().any(|claim| claim["claim_id"].as_str()
                    == Some(REPORT_REPRODUCIBILITY_STANDARD_ID)
                    && claim["phase"].as_str() == Some("run"))
            )
    );
    assert!(
        certificate["pipeline_claims"]
            .as_array()
            .is_some_and(
                |claims| claims.iter().any(|claim| claim["claim_id"].as_str()
                    == Some(VIEW_MODEL_FIDELITY_STANDARD_ID)
                    && claim["phase"].as_str() == Some("view_model"))
            )
    );
    assert_eq!(
        certificate["component_versions"]["belvedere"].as_str(),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        certificate["component_versions"]["amenable"].as_str(),
        Some("0.1.0")
    );
    assert_eq!(
        certificate["component_versions"]["elicitation"].as_str(),
        Some("0.11.1")
    );
    assert_eq!(
        certificate["input_artifact_digests"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().all(|stage_certificate| {
                    stage_certificate["schema_id"].as_str()
                        == Some(BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID)
                        && stage_certificate["schema_version"].as_u64()
                            == Some(u64::from(BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION))
                        && stage_certificate["standards"]
                            .as_array()
                            .is_some_and(|standards| !standards.is_empty())
                        && stage_certificate["requires"]
                            .as_array()
                            .is_some_and(|requirements| !requirements.is_empty())
                        && stage_certificate["ensures"]
                            .as_array()
                            .is_some_and(|guarantees| !guarantees.is_empty())
                        && stage_certificate["input_digests"]
                            .as_array()
                            .is_some_and(|digests| digests.iter().all(is_hex_digest))
                        && stage_certificate["invariant_checks"]
                            .as_array()
                            .is_some_and(|checks| {
                                checks
                                    .iter()
                                    .all(|check| check["passed"].as_bool() == Some(true))
                            })
                })
            )
    );
    assert!(
        certificate["invariant_checks"]
            .as_array()
            .is_some_and(|checks| checks
                .iter()
                .all(|check| check["passed"].as_bool() == Some(true)))
    );
    assert_eq!(certificate["invocation"]["surface"].as_str(), Some("cli"));
    assert_eq!(
        certificate["invocation"]["arguments"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    Ok(())
}

#[test]
fn cli_can_emit_the_normal_run_stage_certificates_as_json() -> Result<()> {
    belvedere::init_tracing();

    let cases = [
        (
            "--registry-ingestion-certificate",
            BelvedereCliArtifact::RunRegistryIngestionCertificate,
            "registry_ingestion",
            "belvedere.ingest.registry_dump",
            5,
        ),
        (
            "--assessment-ingestion-certificate",
            BelvedereCliArtifact::RunAssessmentIngestionCertificate,
            "assessment_ingestion",
            "belvedere.ingest.proof_assessments",
            2,
        ),
        (
            "--landscape-modeling-certificate",
            BelvedereCliArtifact::RunLandscapeModelingCertificate,
            "landscape_modeling",
            "belvedere.model.landscape",
            19,
        ),
        (
            "--auditor-navigation-certificate",
            BelvedereCliArtifact::RunAuditorNavigationCertificate,
            "auditor_navigation",
            "belvedere.view_model.auditor_navigation",
            62,
        ),
        (
            "--ui-ir-projection-certificate",
            BelvedereCliArtifact::RunUiIrProjectionCertificate,
            "ui_ir_projection",
            "belvedere.ui_ir.elicit_ui_projection",
            159,
        ),
    ];

    for (flag, artifact, stage, contract, output_count) in cases {
        let args = [
            flag,
            "--stdio",
            "tests/fixtures/registry_dump.json",
            "tests/fixtures/proof_assessments.jsonl",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err_with(|| format!("{flag} should be part of the public CLI API"))?;

        assert_eq!(command.output(), stdio_output(artifact));

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let certificate: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err_with(|| format!("{flag} output should be parseable JSON"))?;
        assert_eq!(
            certificate["schema_id"].as_str(),
            Some(BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID)
        );
        assert_eq!(
            certificate["schema_version"].as_u64(),
            Some(u64::from(BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION))
        );
        assert!(
            certificate["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("belvedere.stage."))
        );
        assert_eq!(certificate["stage"].as_str(), Some(stage));
        assert_eq!(certificate["contract"].as_str(), Some(contract));
        assert_eq!(
            certificate["output_ids"].as_array().map(Vec::len),
            Some(output_count)
        );
        assert!(
            certificate["invariant_checks"]
                .as_array()
                .is_some_and(|checks| checks
                    .iter()
                    .all(|check| check["passed"].as_bool() == Some(true)))
        );
    }

    Ok(())
}

#[test]
fn cli_can_emit_the_run_manifest_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--manifest",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("manifest JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunManifest)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let manifest: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI manifest output should be parseable JSON")?;
    assert_run_manifest(&manifest);
    Ok(())
}

#[test]
fn cli_can_emit_the_registry_ingestion_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--registry-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("registry receipt JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunRegistryReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI registry receipt output should be parseable JSON")?;
    assert_registry_ingestion_receipt(&receipt);
    Ok(())
}

#[test]
fn cli_can_emit_the_assessment_ingestion_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--assessment-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("assessment receipt JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunAssessmentReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI assessment receipt output should be parseable JSON")?;
    assert_assessment_ingestion_receipt(&receipt);
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--landscape-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("landscape receipt JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunLandscapeReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI landscape receipt output should be parseable JSON")?;
    assert_landscape_receipt(&receipt);
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_graph_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--landscape",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("landscape graph JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunLandscape)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let graph: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI landscape graph output should be parseable JSON")?;
    assert_landscape_graph(&graph);
    Ok(())
}

#[test]
fn cli_can_emit_the_ui_ir_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--ui-ir-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("UI IR receipt JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunUiIrReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI UI IR receipt output should be parseable JSON")?;
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(14));
    assert_eq!(receipt["section_ids"].as_array().map(Vec::len), Some(14));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_HISTORY].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_WORK].as_u64(),
        Some(2)
    );
    assert_eq!(receipt["role_counts"]["role_button"].as_u64(), Some(27));
    assert_eq!(receipt["viewport_ids"].as_array().map(Vec::len), Some(3));
    assert_eq!(
        receipt["trace_source_counts"]["trace_unique_source_records"].as_u64(),
        Some(7)
    );
    assert!(
        receipt["minted_standards"]
            .as_array()
            .is_some_and(|standards| !standards.is_empty())
    );
    Ok(())
}

#[test]
fn cli_can_emit_the_view_model_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--view-model-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("view-model receipt JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::RunViewModelReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI view-model receipt output should be parseable JSON")?;
    assert_auditor_navigation_receipt(&receipt);
    Ok(())
}

#[test]
fn cli_can_emit_run_projection_artifacts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 20] = [
        (
            "--view-model",
            stdio_output(BelvedereCliArtifact::RunViewModel),
            assert_auditor_navigation_model,
        ),
        (
            "--overview",
            stdio_output(BelvedereCliArtifact::RunOverview),
            assert_auditor_overview,
        ),
        (
            "--backend-coverage",
            stdio_output(BelvedereCliArtifact::RunBackendCoverage),
            assert_backend_coverage,
        ),
        (
            "--claim-explorer",
            stdio_output(BelvedereCliArtifact::RunClaimExplorer),
            assert_claim_explorer,
        ),
        (
            "--claim-details",
            stdio_output(BelvedereCliArtifact::RunClaimDetails),
            assert_claim_details,
        ),
        (
            "--filter-facets",
            stdio_output(BelvedereCliArtifact::RunFilterFacets),
            assert_filter_facets,
        ),
        (
            "--source-navigation",
            stdio_output(BelvedereCliArtifact::RunSourceNavigation),
            assert_source_navigation_map,
        ),
        (
            "--proof-chains",
            stdio_output(BelvedereCliArtifact::RunProofChains),
            assert_proof_chains,
        ),
        (
            "--review-workflow",
            stdio_output(BelvedereCliArtifact::RunReviewWorkflow),
            assert_review_workflow_queue,
        ),
        (
            "--assessment-history",
            stdio_output(BelvedereCliArtifact::RunAssessmentHistory),
            assert_assessment_history,
        ),
        (
            "--assessment-queue",
            stdio_output(BelvedereCliArtifact::RunAssessmentQueue),
            assert_assessment_queue,
        ),
        (
            "--assessment-work",
            stdio_output(BelvedereCliArtifact::RunAssessmentWork),
            assert_assessment_work,
        ),
        (
            "--witness-surface",
            stdio_output(BelvedereCliArtifact::RunWitnessSurface),
            assert_witness_surface,
        ),
        (
            "--provenance-roots",
            stdio_output(BelvedereCliArtifact::RunProvenanceRoots),
            assert_provenance_roots,
        ),
        (
            "--gaps",
            stdio_output(BelvedereCliArtifact::RunGaps),
            assert_gap_items,
        ),
        (
            "--ui-ir",
            stdio_output(BelvedereCliArtifact::RunUiIr),
            assert_ui_ir_projection,
        ),
        (
            "--ui-ir-sections",
            stdio_output(BelvedereCliArtifact::RunUiIrSections),
            assert_ui_ir_sections,
        ),
        (
            "--ui-ir-trace-widgets",
            stdio_output(BelvedereCliArtifact::RunUiIrTraceWidgets),
            assert_ui_ir_trace_widgets,
        ),
        (
            "--ui-ir-viewports",
            stdio_output(BelvedereCliArtifact::RunUiIrViewports),
            assert_ui_ir_viewports,
        ),
        (
            "--ui-ir-minted-standards",
            stdio_output(BelvedereCliArtifact::RunUiIrMintedStandards),
            assert_ui_ir_minted_standards,
        ),
    ];

    for (flag, expected_output, assert_artifact) in cases {
        let args = [
            flag,
            "--stdio",
            "tests/fixtures/registry_dump.json",
            "tests/fixtures/proof_assessments.jsonl",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err("projection artifact JSON flag should be part of the public CLI API")?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let artifact: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI projection artifact output should be parseable JSON")?;
        assert_artifact(&artifact);
    }

    Ok(())
}

#[test]
fn cli_can_persist_the_registry_ingestion_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let receipt_path =
        std::env::temp_dir().join(format!("belvedere-registry-receipt-{}.json", process::id()));
    let args = [
        "--registry-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary registry receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("registry receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::RunRegistryReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("registry receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted registry receipt should be parseable JSON")?;
    assert_registry_ingestion_receipt(&receipt);

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary registry receipt file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_assessment_ingestion_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let receipt_path = std::env::temp_dir().join(format!(
        "belvedere-assessment-receipt-{}.json",
        process::id()
    ));
    let args = [
        "--assessment-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("assessment receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::RunAssessmentReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("assessment receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted assessment receipt should be parseable JSON")?;
    assert_assessment_ingestion_receipt(&receipt);

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary assessment receipt file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let receipt_path = std::env::temp_dir().join(format!(
        "belvedere-landscape-receipt-{}.json",
        process::id()
    ));
    let args = [
        "--landscape-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary landscape receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("landscape receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::RunLandscapeReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("landscape receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted landscape receipt should be parseable JSON")?;
    assert_landscape_receipt(&receipt);

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary landscape receipt file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_graph_as_json() -> Result<()> {
    belvedere::init_tracing();

    let graph_path =
        std::env::temp_dir().join(format!("belvedere-landscape-{}.json", process::id()));
    let args = [
        "--landscape",
        "--file",
        graph_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary landscape graph path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("landscape graph file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(BelvedereCliArtifact::RunLandscape, graph_path.clone())
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let graph_bytes = fs::read(&graph_path)
        .into_diagnostic()
        .wrap_err("landscape graph file should be written")?;
    let graph: Value = serde_json::from_slice(&graph_bytes)
        .into_diagnostic()
        .wrap_err("persisted landscape graph should be parseable JSON")?;
    assert_landscape_graph(&graph);

    fs::remove_file(&graph_path)
        .into_diagnostic()
        .wrap_err("temporary landscape graph file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_ui_ir_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let receipt_path =
        std::env::temp_dir().join(format!("belvedere-ui-ir-receipt-{}.json", process::id()));
    let args = [
        "--ui-ir-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary UI IR receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("UI IR receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(BelvedereCliArtifact::RunUiIrReceipt, receipt_path.clone())
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("UI IR receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted UI IR receipt should be parseable JSON")?;
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(14));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_HISTORY].as_u64(),
        Some(2)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_WORK].as_u64(),
        Some(2)
    );

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary UI IR receipt file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_view_model_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let receipt_path = std::env::temp_dir().join(format!(
        "belvedere-view-model-receipt-{}.json",
        process::id()
    ));
    let args = [
        "--view-model-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary view-model receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("view-model receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::RunViewModelReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("view-model receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted view-model receipt should be parseable JSON")?;
    assert_auditor_navigation_receipt(&receipt);

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary view-model receipt file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_run_projection_artifacts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let view_model_path =
        std::env::temp_dir().join(format!("belvedere-view-model-{}.json", process::id()));
    let claim_details_path =
        std::env::temp_dir().join(format!("belvedere-claim-details-{}.json", process::id()));
    let filter_facets_path =
        std::env::temp_dir().join(format!("belvedere-filter-facets-{}.json", process::id()));
    let source_navigation_path = std::env::temp_dir().join(format!(
        "belvedere-source-navigation-{}.json",
        process::id()
    ));
    let proof_chains_path =
        std::env::temp_dir().join(format!("belvedere-proof-chains-{}.json", process::id()));
    let review_workflow_path =
        std::env::temp_dir().join(format!("belvedere-review-workflow-{}.json", process::id()));
    let ui_ir_path = std::env::temp_dir().join(format!("belvedere-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 7] = [
        (
            "--view-model",
            view_model_path.clone(),
            file_output(BelvedereCliArtifact::RunViewModel, view_model_path.clone()),
            assert_auditor_navigation_model,
        ),
        (
            "--claim-details",
            claim_details_path.clone(),
            file_output(
                BelvedereCliArtifact::RunClaimDetails,
                claim_details_path.clone(),
            ),
            assert_claim_details,
        ),
        (
            "--filter-facets",
            filter_facets_path.clone(),
            file_output(
                BelvedereCliArtifact::RunFilterFacets,
                filter_facets_path.clone(),
            ),
            assert_filter_facets,
        ),
        (
            "--source-navigation",
            source_navigation_path.clone(),
            file_output(
                BelvedereCliArtifact::RunSourceNavigation,
                source_navigation_path.clone(),
            ),
            assert_source_navigation_map,
        ),
        (
            "--proof-chains",
            proof_chains_path.clone(),
            file_output(
                BelvedereCliArtifact::RunProofChains,
                proof_chains_path.clone(),
            ),
            assert_proof_chains,
        ),
        (
            "--review-workflow",
            review_workflow_path.clone(),
            file_output(
                BelvedereCliArtifact::RunReviewWorkflow,
                review_workflow_path.clone(),
            ),
            assert_review_workflow_queue,
        ),
        (
            "--ui-ir",
            ui_ir_path.clone(),
            file_output(BelvedereCliArtifact::RunUiIr, ui_ir_path.clone()),
            assert_ui_ir_projection,
        ),
    ];

    for (flag, artifact_path, expected_output, assert_artifact) in cases {
        let artifact_path_text = artifact_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary projection path should be UTF-8"))?;
        let args = [
            flag,
            "--file",
            artifact_path_text,
            "tests/fixtures/registry_dump.json",
            "tests/fixtures/proof_assessments.jsonl",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err("projection artifact file flag should be part of the public CLI API")?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let artifact_bytes = fs::read(&artifact_path)
            .into_diagnostic()
            .wrap_err("projection artifact file should be written")?;
        let artifact: Value = serde_json::from_slice(&artifact_bytes)
            .into_diagnostic()
            .wrap_err("persisted projection artifact should be parseable JSON")?;
        assert_artifact(&artifact);

        fs::remove_file(artifact_path)
            .into_diagnostic()
            .wrap_err("temporary projection artifact file should be removable")?;
    }

    Ok(())
}

#[test]
fn cli_can_persist_the_run_certificate_as_json() -> Result<()> {
    belvedere::init_tracing();

    let certificate_path =
        std::env::temp_dir().join(format!("belvedere-certificate-{}.json", process::id()));
    let args = [
        "--certificate",
        "--file",
        certificate_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary certificate path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("certificate file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::RunCertificate,
            certificate_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let certificate_bytes = fs::read(&certificate_path)
        .into_diagnostic()
        .wrap_err("certificate file should be written")?;
    let certificate: Value = serde_json::from_slice(&certificate_bytes)
        .into_diagnostic()
        .wrap_err("persisted certificate should be parseable JSON")?;
    assert!(
        certificate["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.run:"))
    );
    assert_eq!(
        certificate["schema_id"].as_str(),
        Some(BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID)
    );
    assert_eq!(
        certificate["schema_version"].as_u64(),
        Some(u64::from(BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION))
    );
    assert_eq!(
        certificate["stage_certificates"].as_array().map(Vec::len),
        Some(5)
    );
    assert_eq!(
        certificate["stage_certificate_digests"]
            .as_array()
            .map(Vec::len),
        Some(5)
    );
    assert_eq!(
        certificate["input_artifact_digests"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );
    assert_eq!(
        certificate["component_versions"]["elicit_ui"].as_str(),
        Some("0.11.1")
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().all(|stage_certificate| {
                    stage_certificate["invariant_checks"]
                        .as_array()
                        .is_some_and(|checks| !checks.is_empty())
                        && stage_certificate["standards"]
                            .as_array()
                            .is_some_and(|standards| !standards.is_empty())
                })
            )
    );
    assert!(
        certificate["invariant_checks"]
            .as_array()
            .is_some_and(|checks| !checks.is_empty())
    );

    fs::remove_file(&certificate_path)
        .into_diagnostic()
        .wrap_err("temporary certificate file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_run_manifest_as_json() -> Result<()> {
    belvedere::init_tracing();

    let manifest_path =
        std::env::temp_dir().join(format!("belvedere-manifest-{}.json", process::id()));
    let args = [
        "--manifest",
        "--file",
        manifest_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary manifest path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("manifest file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(BelvedereCliArtifact::RunManifest, manifest_path.clone())
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let manifest_bytes = fs::read(&manifest_path)
        .into_diagnostic()
        .wrap_err("manifest file should be written")?;
    let manifest: Value = serde_json::from_slice(&manifest_bytes)
        .into_diagnostic()
        .wrap_err("persisted manifest should be parseable JSON")?;
    assert_run_manifest(&manifest);

    fs::remove_file(&manifest_path)
        .into_diagnostic()
        .wrap_err("temporary manifest file should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_certificate_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--certificate",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff certificate flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffCertificate)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let certificate: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff certificate output should be parseable JSON")?;
    assert!(
        certificate["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.stage.landscape_diff:"))
    );
    assert_eq!(certificate["stage"].as_str(), Some("landscape_diff"));
    assert_eq!(
        certificate["contract"].as_str(),
        Some("belvedere.diff.landscape")
    );
    assert_eq!(certificate["counts"]["total_items"].as_u64(), Some(6));
    assert_eq!(
        certificate["counts"]["diff_item_kind_added"].as_u64(),
        Some(4)
    );
    assert_eq!(
        certificate["counts"]["diff_item_kind_changed"].as_u64(),
        Some(1)
    );
    assert_eq!(
        certificate["counts"]["diff_item_kind_removed"].as_u64(),
        Some(1)
    );
    assert_eq!(
        certificate["counts"]["diff_review_priority_review"].as_u64(),
        Some(4)
    );
    assert_eq!(
        certificate["counts"]["diff_review_action_triage_new_gap"].as_u64(),
        Some(1)
    );
    assert_eq!(
        certificate["counts"]["diff_trace_source_links"].as_u64(),
        Some(8)
    );
    assert_eq!(
        certificate["counts"]["diff_trace_unique_source_records"].as_u64(),
        Some(3)
    );
    assert!(
        certificate["invariant_checks"]
            .as_array()
            .is_some_and(|checks| {
                checks
                    .iter()
                    .all(|check| check["passed"].as_bool() == Some(true))
            })
    );

    fs::remove_file(&previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_review_stage_certificates_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-review-certificate-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;
    let previous_assessments = previous_assessments_path
        .to_str()
        .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?;

    let cases = [
        (
            "--diff-review-certificate",
            BelvedereCliArtifact::LandscapeDiffReviewCertificate,
            "landscape_diff_review",
            "belvedere.view_model.landscape_diff_review",
            "total_items",
            6,
        ),
        (
            "--diff-review-ui-ir-certificate",
            BelvedereCliArtifact::LandscapeDiffReviewUiIrCertificate,
            "landscape_diff_review_ui_ir_projection",
            "belvedere.ui_ir.elicit_ui_diff_review_projection",
            "ui_ir_sections_populated",
            3,
        ),
    ];

    for (flag, artifact, stage, contract, count_key, count) in cases {
        let args = [
            "diff",
            flag,
            "--stdio",
            "tests/fixtures/registry_dump.json",
            previous_assessments,
            "tests/fixtures/registry_dump.json",
            "tests/fixtures/proof_assessments.jsonl",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<Vec<_>>();
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err_with(|| format!("{flag} should be part of the public CLI API"))?;

        assert_eq!(command.output(), stdio_output(artifact));

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let certificate: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err_with(|| format!("{flag} output should be parseable JSON"))?;
        assert!(
            certificate["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("belvedere.stage."))
        );
        assert_eq!(certificate["stage"].as_str(), Some(stage));
        assert_eq!(certificate["contract"].as_str(), Some(contract));
        assert_eq!(certificate["counts"][count_key].as_u64(), Some(count));
        assert!(
            certificate["invariant_checks"]
                .as_array()
                .is_some_and(|checks| checks
                    .iter()
                    .all(|check| check["passed"].as_bool() == Some(true)))
        );
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_diff_snapshot_stage_certificates_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-snapshot-certificate-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases = [
        (
            "--previous-registry-ingestion-certificate",
            BelvedereCliArtifact::LandscapeDiffPreviousRegistryIngestionCertificate,
            "registry_ingestion",
            "belvedere.ingest.registry_dump",
            "proof_records",
            1,
            None,
        ),
        (
            "--previous-assessment-ingestion-certificate",
            BelvedereCliArtifact::LandscapeDiffPreviousAssessmentIngestionCertificate,
            "assessment_ingestion",
            "belvedere.ingest.proof_assessments",
            "assessments",
            0,
            None,
        ),
        (
            "--previous-landscape-modeling-certificate",
            BelvedereCliArtifact::LandscapeDiffPreviousLandscapeModelingCertificate,
            "landscape_modeling",
            "belvedere.model.landscape",
            "claims",
            3,
            None,
        ),
        (
            "--previous-auditor-navigation-certificate",
            BelvedereCliArtifact::LandscapeDiffPreviousAuditorNavigationCertificate,
            "auditor_navigation",
            "belvedere.view_model.auditor_navigation",
            "view_model_items",
            44,
            None,
        ),
        (
            "--previous-ui-ir-projection-certificate",
            BelvedereCliArtifact::LandscapeDiffPreviousUiIrProjectionCertificate,
            "ui_ir_projection",
            "belvedere.ui_ir.elicit_ui_projection",
            "widgets",
            44,
            Some("ui_ir_sections_traceable"),
        ),
        (
            "--current-registry-ingestion-certificate",
            BelvedereCliArtifact::LandscapeDiffCurrentRegistryIngestionCertificate,
            "registry_ingestion",
            "belvedere.ingest.registry_dump",
            "proof_records",
            1,
            None,
        ),
        (
            "--current-assessment-ingestion-certificate",
            BelvedereCliArtifact::LandscapeDiffCurrentAssessmentIngestionCertificate,
            "assessment_ingestion",
            "belvedere.ingest.proof_assessments",
            "assessments",
            2,
            None,
        ),
        (
            "--current-landscape-modeling-certificate",
            BelvedereCliArtifact::LandscapeDiffCurrentLandscapeModelingCertificate,
            "landscape_modeling",
            "belvedere.model.landscape",
            "claims",
            4,
            None,
        ),
        (
            "--current-auditor-navigation-certificate",
            BelvedereCliArtifact::LandscapeDiffCurrentAuditorNavigationCertificate,
            "auditor_navigation",
            "belvedere.view_model.auditor_navigation",
            "view_model_items",
            62,
            None,
        ),
        (
            "--current-ui-ir-projection-certificate",
            BelvedereCliArtifact::LandscapeDiffCurrentUiIrProjectionCertificate,
            "ui_ir_projection",
            "belvedere.ui_ir.elicit_ui_projection",
            "widgets",
            62,
            None,
        ),
    ];

    for (flag, artifact, stage, contract, count_key, count, expected_failed_invariant) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err_with(|| format!("{flag} should be part of the public CLI API"))?;

        assert_eq!(command.output(), stdio_output(artifact));

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let certificate: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err_with(|| format!("{flag} output should be parseable JSON"))?;
        assert_eq!(
            certificate["schema_id"].as_str(),
            Some(BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID)
        );
        assert_eq!(
            certificate["schema_version"].as_u64(),
            Some(u64::from(BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION))
        );
        assert_eq!(certificate["stage"].as_str(), Some(stage));
        assert_eq!(certificate["contract"].as_str(), Some(contract));
        assert_eq!(certificate["counts"][count_key].as_u64(), Some(count));
        let invariant_checks = certificate["invariant_checks"]
            .as_array()
            .ok_or_else(|| miette::miette!("{flag} should emit invariant checks"))?;
        let failed_invariants = invariant_checks
            .iter()
            .filter_map(|check| {
                (check["passed"].as_bool() == Some(false))
                    .then(|| check["name"].as_str())
                    .flatten()
            })
            .collect::<Vec<_>>();

        match expected_failed_invariant {
            Some(invariant) => assert_eq!(failed_invariants, vec![invariant]),
            None => assert!(
                failed_invariants.is_empty(),
                "{flag} should not report failed invariants: {failed_invariants:?}"
            ),
        }
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_run_certificate_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-run-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--run-certificate",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff run certificate flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffRunCertificate)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let certificate: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff run certificate output should be parseable JSON")?;
    assert!(
        certificate["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.run:"))
    );
    assert_eq!(
        certificate["schema_id"].as_str(),
        Some(BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID)
    );
    assert_eq!(
        certificate["stage_certificates"].as_array().map(Vec::len),
        Some(13)
    );
    assert_eq!(
        certificate["stage_certificate_digests"]
            .as_array()
            .map(Vec::len),
        Some(13)
    );
    assert_eq!(
        certificate["input_artifact_ids"].as_array().map(Vec::len),
        Some(4)
    );
    assert_eq!(
        certificate["invocation"]["arguments"]
            .as_array()
            .map(Vec::len),
        Some(7)
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().any(|stage_certificate| {
                    stage_certificate["stage"].as_str() == Some("landscape_diff")
                        && stage_certificate["contract"].as_str()
                            == Some("belvedere.diff.landscape")
                        && stage_certificate["counts"]["total_items"].as_u64() == Some(6)
                        && stage_certificate["counts"]["diff_item_kind_added"].as_u64() == Some(4)
                        && stage_certificate["counts"]["diff_trace_source_links"].as_u64()
                            == Some(8)
                        && stage_certificate["counts"]["source_landscape_receipts"].as_u64()
                            == Some(2)
                        && stage_certificate["input_digests"]
                            .as_array()
                            .is_some_and(|digests| {
                                digests.len() == 2 && digests.iter().all(is_hex_digest)
                            })
                        && stage_certificate["invariant_checks"]
                            .as_array()
                            .is_some_and(|checks| {
                                checks.iter().any(|check| {
                                    check["name"].as_str()
                                        == Some("landscape_diff_trace_sources_admissible")
                                        && check["passed"].as_bool() == Some(true)
                                })
                            })
                })
            )
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().any(|stage_certificate| {
                    stage_certificate["stage"].as_str()
                        == Some("landscape_diff_review_ui_ir_projection")
                        && stage_certificate["contract"].as_str()
                            == Some("belvedere.ui_ir.elicit_ui_diff_review_projection")
                        && stage_certificate["counts"]["ui_ir_sections_populated"].as_u64()
                            == Some(3)
                        && stage_certificate["counts"]["ui_ir_source_navigation_items"].as_u64()
                            == Some(7)
                })
            )
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().any(|stage_certificate| {
                    stage_certificate["stage"].as_str() == Some("landscape_diff_review")
                        && stage_certificate["contract"].as_str()
                            == Some("belvedere.view_model.landscape_diff_review")
                        && stage_certificate["counts"]["total_items"].as_u64() == Some(6)
                        && stage_certificate["counts"]["diff_review_priority_blocking"].as_u64()
                            == Some(1)
                        && stage_certificate["input_ids"]
                            .as_array()
                            .is_some_and(|ids| ids.len() == 1)
                        && stage_certificate["invariant_checks"]
                            .as_array()
                            .is_some_and(|checks| {
                                checks.iter().any(|check| {
                                    check["name"].as_str()
                                        == Some("landscape_diff_review_actions_classified")
                                        && check["passed"].as_bool() == Some(true)
                                })
                            })
                })
            )
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().any(|stage_certificate| {
                    stage_certificate["stage"].as_str() == Some("ui_ir_projection")
                        && stage_certificate["counts"]["ui_ir_sections_populated"].as_u64()
                            == Some(14)
                })
            )
    );

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_manifest_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-manifest-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--manifest",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff manifest flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffManifest)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let manifest: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff manifest output should be parseable JSON")?;
    assert_diff_manifest(&manifest);

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_diff_run_certificate_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-run-file-empty-assessments-{}.jsonl",
        process::id()
    ));
    let certificate_path = std::env::temp_dir().join(format!(
        "belvedere-diff-run-certificate-{}.json",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--run-certificate",
        "--file",
        certificate_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary certificate path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff run certificate file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::LandscapeDiffRunCertificate,
            certificate_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let certificate_bytes = fs::read(&certificate_path)
        .into_diagnostic()
        .wrap_err("diff run certificate file should be written")?;
    let certificate: Value = serde_json::from_slice(&certificate_bytes)
        .into_diagnostic()
        .wrap_err("persisted diff run certificate should be parseable JSON")?;
    assert!(
        certificate["id"]
            .as_str()
            .is_some_and(|id| id.starts_with("belvedere.run:"))
    );
    assert_eq!(
        certificate["schema_id"].as_str(),
        Some(BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID)
    );
    assert_eq!(
        certificate["stage_certificates"].as_array().map(Vec::len),
        Some(13)
    );
    assert_eq!(
        certificate["input_artifact_digests"]
            .as_array()
            .map(Vec::len),
        Some(4)
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().any(|stage_certificate| {
                    stage_certificate["stage"].as_str() == Some("landscape_diff")
                        && stage_certificate["invariant_checks"]
                            .as_array()
                            .is_some_and(|checks| !checks.is_empty())
                })
            )
    );
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().any(|stage_certificate| {
                    stage_certificate["stage"].as_str() == Some("auditor_navigation")
                        && stage_certificate["counts"]["review_workflow_task_total"].as_u64()
                            == Some(4)
                })
            )
    );

    fs::remove_file(&certificate_path)
        .into_diagnostic()
        .wrap_err("temporary diff run certificate file should be removable")?;
    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-receipt-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--diff-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff receipt flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff receipt output should be parseable JSON")?;
    assert_landscape_diff_receipt(&receipt);

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_diff_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-receipt-file-empty-assessments-{}.jsonl",
        process::id()
    ));
    let receipt_path =
        std::env::temp_dir().join(format!("belvedere-diff-receipt-{}.json", process::id()));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--diff-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary diff receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::LandscapeDiffReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("diff receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted diff receipt should be parseable JSON")?;
    assert_landscape_diff_receipt(&receipt);

    fs::remove_file(receipt_path)
        .into_diagnostic()
        .wrap_err("temporary diff receipt file should be removable")?;
    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_diff_snapshot_ingestion_receipts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-snapshot-ingestion-receipts-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 4] = [
        (
            "--previous-registry-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousRegistryReceipt),
            assert_registry_ingestion_receipt,
        ),
        (
            "--previous-assessment-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentReceipt),
            assert_empty_assessment_ingestion_receipt,
        ),
        (
            "--current-registry-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentRegistryReceipt),
            assert_registry_ingestion_receipt,
        ),
        (
            "--current-assessment-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentReceipt),
            assert_assessment_ingestion_receipt,
        ),
    ];

    for (flag, expected_output, assert_receipt) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "diff snapshot ingestion receipt flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let receipt: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI diff snapshot ingestion receipt output should be parseable JSON")?;
        assert_receipt(&receipt);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_current_diff_registry_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-current-registry-receipt-empty-assessments-{}.jsonl",
        process::id()
    ));
    let receipt_path = std::env::temp_dir().join(format!(
        "belvedere-current-registry-receipt-{}.json",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let receipt_path_text = receipt_path.to_str().ok_or_else(|| {
        miette::miette!("temporary current registry receipt path should be UTF-8")
    })?;
    let args = diff_args_with_empty_previous_assessments(
        &["--current-registry-receipt", receipt_path_text],
        &previous_assessments_path,
    )?;
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("current registry receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::LandscapeDiffCurrentRegistryReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("current registry receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted current registry receipt should be parseable JSON")?;
    assert_registry_ingestion_receipt(&receipt);

    fs::remove_file(receipt_path)
        .into_diagnostic()
        .wrap_err("temporary current registry receipt file should be removable")?;
    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_diff_snapshot_landscape_receipts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-snapshot-landscape-receipts-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-landscape-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousLandscapeReceipt),
            assert_previous_landscape_receipt,
        ),
        (
            "--current-landscape-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentLandscapeReceipt),
            assert_landscape_receipt,
        ),
    ];

    for (flag, expected_output, assert_receipt) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "diff snapshot landscape receipt flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let receipt: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI diff snapshot landscape receipt output should be parseable JSON")?;
        assert_receipt(&receipt);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_diff_snapshot_landscapes_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-snapshot-landscapes-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-landscape",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousLandscape),
            assert_previous_landscape_graph,
        ),
        (
            "--current-landscape",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentLandscape),
            assert_landscape_graph,
        ),
    ];

    for (flag, expected_output, assert_graph) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err("diff snapshot landscape flag should be part of the public CLI API")?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let graph: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI diff snapshot landscape output should be parseable JSON")?;
        assert_graph(&graph);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_diff_snapshot_landscape_receipts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-snapshot-landscape-receipt-files-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let previous_receipt_path = std::env::temp_dir().join(format!(
        "belvedere-previous-landscape-receipt-{}.json",
        process::id()
    ));
    let current_receipt_path = std::env::temp_dir().join(format!(
        "belvedere-current-landscape-receipt-{}.json",
        process::id()
    ));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-landscape-receipt",
            previous_receipt_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousLandscapeReceipt,
                previous_receipt_path.clone(),
            ),
            assert_previous_landscape_receipt,
        ),
        (
            "--current-landscape-receipt",
            current_receipt_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentLandscapeReceipt,
                current_receipt_path.clone(),
            ),
            assert_landscape_receipt,
        ),
    ];

    for (flag, receipt_path, expected_output, assert_receipt) in cases {
        let receipt_path_text = receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary landscape receipt path should be UTF-8"))?;
        let args = diff_args_with_empty_previous_assessments(
            &[flag, receipt_path_text],
            &previous_assessments_path,
        )?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "diff snapshot landscape receipt file flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let receipt_bytes = fs::read(&receipt_path)
            .into_diagnostic()
            .wrap_err("snapshot landscape receipt file should be written")?;
        let receipt: Value = serde_json::from_slice(&receipt_bytes)
            .into_diagnostic()
            .wrap_err("persisted snapshot landscape receipt should be parseable JSON")?;
        assert_receipt(&receipt);

        fs::remove_file(receipt_path)
            .into_diagnostic()
            .wrap_err("temporary snapshot landscape receipt file should be removable")?;
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_diff_snapshot_landscapes_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-snapshot-landscape-files-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let previous_graph_path = std::env::temp_dir().join(format!(
        "belvedere-previous-landscape-{}.json",
        process::id()
    ));
    let current_graph_path = std::env::temp_dir().join(format!(
        "belvedere-current-landscape-{}.json",
        process::id()
    ));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-landscape",
            previous_graph_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousLandscape,
                previous_graph_path.clone(),
            ),
            assert_previous_landscape_graph,
        ),
        (
            "--current-landscape",
            current_graph_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentLandscape,
                current_graph_path.clone(),
            ),
            assert_landscape_graph,
        ),
    ];

    for (flag, graph_path, expected_output, assert_graph) in cases {
        let graph_path_text = graph_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary landscape graph path should be UTF-8"))?;
        let args = diff_args_with_empty_previous_assessments(
            &[flag, graph_path_text],
            &previous_assessments_path,
        )?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err("diff snapshot landscape file flag should be part of the public CLI API")?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let graph_bytes = fs::read(&graph_path)
            .into_diagnostic()
            .wrap_err("snapshot landscape graph file should be written")?;
        let graph: Value = serde_json::from_slice(&graph_bytes)
            .into_diagnostic()
            .wrap_err("persisted snapshot landscape graph should be parseable JSON")?;
        assert_graph(&graph);

        fs::remove_file(graph_path)
            .into_diagnostic()
            .wrap_err("temporary snapshot landscape graph file should be removable")?;
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_landscape_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-landscape-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--landscape-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff landscape receipt flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffLandscapeReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff landscape receipt output should be parseable JSON")?;
    assert_landscape_receipt(&receipt);

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_diff_landscape_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-landscape-file-empty-assessments-{}.jsonl",
        process::id()
    ));
    let receipt_path = std::env::temp_dir().join(format!(
        "belvedere-diff-landscape-receipt-{}.json",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--landscape-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary landscape receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff landscape receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::LandscapeDiffLandscapeReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("diff landscape receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted diff landscape receipt should be parseable JSON")?;
    assert_landscape_receipt(&receipt);

    fs::remove_file(receipt_path)
        .into_diagnostic()
        .wrap_err("temporary landscape receipt file should be removable")?;
    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_landscape_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-landscape-graph-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--landscape",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff landscape graph flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffLandscape)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let graph: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff landscape graph output should be parseable JSON")?;
    assert_landscape_graph(&graph);

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_diff_landscape_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-landscape-graph-file-empty-assessments-{}.jsonl",
        process::id()
    ));
    let graph_path =
        std::env::temp_dir().join(format!("belvedere-diff-landscape-{}.json", process::id()));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--landscape",
        "--file",
        graph_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary landscape graph path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff landscape graph file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::LandscapeDiffLandscape,
            graph_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let graph_bytes = fs::read(&graph_path)
        .into_diagnostic()
        .wrap_err("diff landscape graph file should be written")?;
    let graph: Value = serde_json::from_slice(&graph_bytes)
        .into_diagnostic()
        .wrap_err("persisted diff landscape graph should be parseable JSON")?;
    assert_landscape_graph(&graph);

    fs::remove_file(graph_path)
        .into_diagnostic()
        .wrap_err("temporary landscape graph file should be removable")?;
    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_previous_diff_projection_receipts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-previous-diff-projection-receipts-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-view-model-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousViewModelReceipt),
            assert_previous_auditor_navigation_receipt,
        ),
        (
            "--previous-ui-ir-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousUiIrReceipt),
            assert_previous_ui_ir_receipt,
        ),
    ];

    for (flag, expected_output, assert_receipt) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "previous diff projection receipt flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let receipt: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI previous diff projection receipt output should be parseable JSON")?;
        assert_receipt(&receipt);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_previous_diff_projection_receipts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-previous-diff-projection-receipt-files-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let view_model_receipt_path = std::env::temp_dir().join(format!(
        "belvedere-previous-view-model-receipt-{}.json",
        process::id()
    ));
    let ui_ir_receipt_path = std::env::temp_dir().join(format!(
        "belvedere-previous-ui-ir-receipt-{}.json",
        process::id()
    ));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-view-model-receipt",
            view_model_receipt_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousViewModelReceipt,
                view_model_receipt_path.clone(),
            ),
            assert_previous_auditor_navigation_receipt,
        ),
        (
            "--previous-ui-ir-receipt",
            ui_ir_receipt_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousUiIrReceipt,
                ui_ir_receipt_path.clone(),
            ),
            assert_previous_ui_ir_receipt,
        ),
    ];

    for (flag, receipt_path, expected_output, assert_receipt) in cases {
        let receipt_path_text = receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary projection receipt path should be UTF-8"))?;
        let args = diff_args_with_empty_previous_assessments(
            &[flag, receipt_path_text],
            &previous_assessments_path,
        )?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "previous diff projection receipt file flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let receipt_bytes = fs::read(&receipt_path)
            .into_diagnostic()
            .wrap_err("previous diff projection receipt file should be written")?;
        let receipt: Value = serde_json::from_slice(&receipt_bytes)
            .into_diagnostic()
            .wrap_err("persisted previous diff projection receipt should be parseable JSON")?;
        assert_receipt(&receipt);

        fs::remove_file(receipt_path)
            .into_diagnostic()
            .wrap_err("temporary previous diff projection receipt file should be removable")?;
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_previous_diff_projection_artifacts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-previous-diff-projection-artifacts-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 20] = [
        (
            "--previous-view-model",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousViewModel),
            assert_previous_auditor_navigation_model,
        ),
        (
            "--previous-overview",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousOverview),
            assert_previous_auditor_overview,
        ),
        (
            "--previous-backend-coverage",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousBackendCoverage),
            assert_previous_backend_coverage,
        ),
        (
            "--previous-claim-explorer",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousClaimExplorer),
            assert_previous_claim_explorer,
        ),
        (
            "--previous-claim-details",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousClaimDetails),
            assert_previous_claim_details,
        ),
        (
            "--previous-filter-facets",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousFilterFacets),
            assert_previous_filter_facets,
        ),
        (
            "--previous-source-navigation",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousSourceNavigation),
            assert_previous_source_navigation_map,
        ),
        (
            "--previous-proof-chains",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousProofChains),
            assert_previous_proof_chains,
        ),
        (
            "--previous-review-workflow",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousReviewWorkflow),
            assert_previous_review_workflow_queue,
        ),
        (
            "--previous-assessment-history",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentHistory),
            assert_empty_assessment_history,
        ),
        (
            "--previous-assessment-queue",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentQueue),
            assert_empty_assessment_queue,
        ),
        (
            "--previous-assessment-work",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousAssessmentWork),
            assert_assessment_work,
        ),
        (
            "--previous-witness-surface",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousWitnessSurface),
            assert_witness_surface,
        ),
        (
            "--previous-provenance-roots",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousProvenanceRoots),
            assert_previous_provenance_roots,
        ),
        (
            "--previous-gaps",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousGaps),
            assert_previous_gap_items,
        ),
        (
            "--previous-ui-ir",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousUiIr),
            assert_previous_ui_ir_projection,
        ),
        (
            "--previous-ui-ir-sections",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousUiIrSections),
            assert_ui_ir_sections,
        ),
        (
            "--previous-ui-ir-trace-widgets",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousUiIrTraceWidgets),
            assert_previous_ui_ir_trace_widgets,
        ),
        (
            "--previous-ui-ir-viewports",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousUiIrViewports),
            assert_ui_ir_viewports,
        ),
        (
            "--previous-ui-ir-minted-standards",
            stdio_output(BelvedereCliArtifact::LandscapeDiffPreviousUiIrMintedStandards),
            assert_ui_ir_minted_standards,
        ),
    ];

    for (flag, expected_output, assert_artifact) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "previous diff projection artifact flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let artifact: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI previous diff projection artifact output should be parseable JSON")?;
        assert_artifact(&artifact);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_previous_diff_projection_artifacts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-previous-diff-projection-artifact-files-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let view_model_path = std::env::temp_dir().join(format!(
        "belvedere-previous-view-model-{}.json",
        process::id()
    ));
    let claim_details_path = std::env::temp_dir().join(format!(
        "belvedere-previous-claim-details-{}.json",
        process::id()
    ));
    let filter_facets_path = std::env::temp_dir().join(format!(
        "belvedere-previous-filter-facets-{}.json",
        process::id()
    ));
    let source_navigation_path = std::env::temp_dir().join(format!(
        "belvedere-previous-source-navigation-{}.json",
        process::id()
    ));
    let proof_chains_path = std::env::temp_dir().join(format!(
        "belvedere-previous-proof-chains-{}.json",
        process::id()
    ));
    let ui_ir_path =
        std::env::temp_dir().join(format!("belvedere-previous-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 6] = [
        (
            "--previous-view-model",
            view_model_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousViewModel,
                view_model_path.clone(),
            ),
            assert_previous_auditor_navigation_model,
        ),
        (
            "--previous-claim-details",
            claim_details_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousClaimDetails,
                claim_details_path.clone(),
            ),
            assert_previous_claim_details,
        ),
        (
            "--previous-filter-facets",
            filter_facets_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousFilterFacets,
                filter_facets_path.clone(),
            ),
            assert_previous_filter_facets,
        ),
        (
            "--previous-source-navigation",
            source_navigation_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousSourceNavigation,
                source_navigation_path.clone(),
            ),
            assert_previous_source_navigation_map,
        ),
        (
            "--previous-proof-chains",
            proof_chains_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousProofChains,
                proof_chains_path.clone(),
            ),
            assert_previous_proof_chains,
        ),
        (
            "--previous-ui-ir",
            ui_ir_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffPreviousUiIr,
                ui_ir_path.clone(),
            ),
            assert_previous_ui_ir_projection,
        ),
    ];

    for (flag, artifact_path, expected_output, assert_artifact) in cases {
        let artifact_path_text = artifact_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary projection artifact path should be UTF-8"))?;
        let args = diff_args_with_empty_previous_assessments(
            &[flag, artifact_path_text],
            &previous_assessments_path,
        )?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "previous diff projection artifact file flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let artifact_bytes = fs::read(&artifact_path)
            .into_diagnostic()
            .wrap_err("previous diff projection artifact file should be written")?;
        let artifact: Value = serde_json::from_slice(&artifact_bytes)
            .into_diagnostic()
            .wrap_err("persisted previous diff projection artifact should be parseable JSON")?;
        assert_artifact(&artifact);

        fs::remove_file(artifact_path)
            .into_diagnostic()
            .wrap_err("temporary previous diff projection artifact file should be removable")?;
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_current_diff_projection_receipts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-current-diff-projection-receipts-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--current-view-model-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentViewModelReceipt),
            assert_auditor_navigation_receipt,
        ),
        (
            "--current-ui-ir-receipt",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentUiIrReceipt),
            assert_ui_ir_receipt,
        ),
    ];

    for (flag, expected_output, assert_receipt) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "current diff projection receipt flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let receipt: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI current diff projection receipt output should be parseable JSON")?;
        assert_receipt(&receipt);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_current_diff_projection_receipts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-current-diff-projection-receipt-files-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let view_model_receipt_path = std::env::temp_dir().join(format!(
        "belvedere-current-view-model-receipt-{}.json",
        process::id()
    ));
    let ui_ir_receipt_path = std::env::temp_dir().join(format!(
        "belvedere-current-ui-ir-receipt-{}.json",
        process::id()
    ));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--current-view-model-receipt",
            view_model_receipt_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentViewModelReceipt,
                view_model_receipt_path.clone(),
            ),
            assert_auditor_navigation_receipt,
        ),
        (
            "--current-ui-ir-receipt",
            ui_ir_receipt_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentUiIrReceipt,
                ui_ir_receipt_path.clone(),
            ),
            assert_ui_ir_receipt,
        ),
    ];

    for (flag, receipt_path, expected_output, assert_receipt) in cases {
        let receipt_path_text = receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary projection receipt path should be UTF-8"))?;
        let args = diff_args_with_empty_previous_assessments(
            &[flag, receipt_path_text],
            &previous_assessments_path,
        )?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "current diff projection receipt file flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let receipt_bytes = fs::read(&receipt_path)
            .into_diagnostic()
            .wrap_err("current diff projection receipt file should be written")?;
        let receipt: Value = serde_json::from_slice(&receipt_bytes)
            .into_diagnostic()
            .wrap_err("persisted current diff projection receipt should be parseable JSON")?;
        assert_receipt(&receipt);

        fs::remove_file(receipt_path)
            .into_diagnostic()
            .wrap_err("temporary current diff projection receipt file should be removable")?;
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_current_diff_projection_artifacts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-current-diff-projection-artifacts-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 20] = [
        (
            "--current-view-model",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentViewModel),
            assert_auditor_navigation_model,
        ),
        (
            "--current-overview",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentOverview),
            assert_auditor_overview,
        ),
        (
            "--current-backend-coverage",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentBackendCoverage),
            assert_backend_coverage,
        ),
        (
            "--current-claim-explorer",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentClaimExplorer),
            assert_claim_explorer,
        ),
        (
            "--current-claim-details",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentClaimDetails),
            assert_claim_details,
        ),
        (
            "--current-filter-facets",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentFilterFacets),
            assert_filter_facets,
        ),
        (
            "--current-source-navigation",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentSourceNavigation),
            assert_source_navigation_map,
        ),
        (
            "--current-proof-chains",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentProofChains),
            assert_proof_chains,
        ),
        (
            "--current-review-workflow",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentReviewWorkflow),
            assert_review_workflow_queue,
        ),
        (
            "--current-assessment-history",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentHistory),
            assert_assessment_history,
        ),
        (
            "--current-assessment-queue",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentQueue),
            assert_assessment_queue,
        ),
        (
            "--current-assessment-work",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentAssessmentWork),
            assert_assessment_work,
        ),
        (
            "--current-witness-surface",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentWitnessSurface),
            assert_witness_surface,
        ),
        (
            "--current-provenance-roots",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentProvenanceRoots),
            assert_provenance_roots,
        ),
        (
            "--current-gaps",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentGaps),
            assert_gap_items,
        ),
        (
            "--current-ui-ir",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentUiIr),
            assert_ui_ir_projection,
        ),
        (
            "--current-ui-ir-sections",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentUiIrSections),
            assert_ui_ir_sections,
        ),
        (
            "--current-ui-ir-trace-widgets",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentUiIrTraceWidgets),
            assert_ui_ir_trace_widgets,
        ),
        (
            "--current-ui-ir-viewports",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentUiIrViewports),
            assert_ui_ir_viewports,
        ),
        (
            "--current-ui-ir-minted-standards",
            stdio_output(BelvedereCliArtifact::LandscapeDiffCurrentUiIrMintedStandards),
            assert_ui_ir_minted_standards,
        ),
    ];

    for (flag, expected_output, assert_artifact) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "current diff projection artifact flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let artifact: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI current diff projection artifact output should be parseable JSON")?;
        assert_artifact(&artifact);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_current_diff_projection_artifacts_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-current-diff-projection-artifact-files-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let view_model_path = std::env::temp_dir().join(format!(
        "belvedere-current-view-model-{}.json",
        process::id()
    ));
    let claim_details_path = std::env::temp_dir().join(format!(
        "belvedere-current-claim-details-{}.json",
        process::id()
    ));
    let filter_facets_path = std::env::temp_dir().join(format!(
        "belvedere-current-filter-facets-{}.json",
        process::id()
    ));
    let source_navigation_path = std::env::temp_dir().join(format!(
        "belvedere-current-source-navigation-{}.json",
        process::id()
    ));
    let proof_chains_path = std::env::temp_dir().join(format!(
        "belvedere-current-proof-chains-{}.json",
        process::id()
    ));
    let ui_ir_path =
        std::env::temp_dir().join(format!("belvedere-current-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 6] = [
        (
            "--current-view-model",
            view_model_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentViewModel,
                view_model_path.clone(),
            ),
            assert_auditor_navigation_model,
        ),
        (
            "--current-claim-details",
            claim_details_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentClaimDetails,
                claim_details_path.clone(),
            ),
            assert_claim_details,
        ),
        (
            "--current-filter-facets",
            filter_facets_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentFilterFacets,
                filter_facets_path.clone(),
            ),
            assert_filter_facets,
        ),
        (
            "--current-source-navigation",
            source_navigation_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentSourceNavigation,
                source_navigation_path.clone(),
            ),
            assert_source_navigation_map,
        ),
        (
            "--current-proof-chains",
            proof_chains_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentProofChains,
                proof_chains_path.clone(),
            ),
            assert_proof_chains,
        ),
        (
            "--current-ui-ir",
            ui_ir_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffCurrentUiIr,
                ui_ir_path.clone(),
            ),
            assert_ui_ir_projection,
        ),
    ];

    for (flag, artifact_path, expected_output, assert_artifact) in cases {
        let artifact_path_text = artifact_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary projection artifact path should be UTF-8"))?;
        let args = diff_args_with_empty_previous_assessments(
            &[flag, artifact_path_text],
            &previous_assessments_path,
        )?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err(
                "current diff projection artifact file flag should be part of the public CLI API",
            )?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let artifact_bytes = fs::read(&artifact_path)
            .into_diagnostic()
            .wrap_err("current diff projection artifact file should be written")?;
        let artifact: Value = serde_json::from_slice(&artifact_bytes)
            .into_diagnostic()
            .wrap_err("persisted current diff projection artifact should be parseable JSON")?;
        assert_artifact(&artifact);

        fs::remove_file(artifact_path)
            .into_diagnostic()
            .wrap_err("temporary current diff projection artifact file should be removable")?;
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_landscape_diff_projection_artifact_aliases_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-projection-artifact-aliases-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 20] = [
        (
            "--view-model",
            stdio_output(BelvedereCliArtifact::LandscapeDiffViewModel),
            assert_landscape_diff_review_model,
        ),
        (
            "--overview",
            stdio_output(BelvedereCliArtifact::LandscapeDiffOverview),
            assert_auditor_overview,
        ),
        (
            "--backend-coverage",
            stdio_output(BelvedereCliArtifact::LandscapeDiffBackendCoverage),
            assert_backend_coverage,
        ),
        (
            "--claim-explorer",
            stdio_output(BelvedereCliArtifact::LandscapeDiffClaimExplorer),
            assert_claim_explorer,
        ),
        (
            "--claim-details",
            stdio_output(BelvedereCliArtifact::LandscapeDiffClaimDetails),
            assert_claim_details,
        ),
        (
            "--filter-facets",
            stdio_output(BelvedereCliArtifact::LandscapeDiffFilterFacets),
            assert_filter_facets,
        ),
        (
            "--source-navigation",
            stdio_output(BelvedereCliArtifact::LandscapeDiffSourceNavigation),
            assert_source_navigation_map,
        ),
        (
            "--proof-chains",
            stdio_output(BelvedereCliArtifact::LandscapeDiffProofChains),
            assert_proof_chains,
        ),
        (
            "--review-workflow",
            stdio_output(BelvedereCliArtifact::LandscapeDiffReviewWorkflow),
            assert_landscape_diff_review_workflow_queue,
        ),
        (
            "--assessment-history",
            stdio_output(BelvedereCliArtifact::LandscapeDiffAssessmentHistory),
            assert_assessment_history,
        ),
        (
            "--assessment-queue",
            stdio_output(BelvedereCliArtifact::LandscapeDiffAssessmentQueue),
            assert_assessment_queue,
        ),
        (
            "--assessment-work",
            stdio_output(BelvedereCliArtifact::LandscapeDiffAssessmentWork),
            assert_assessment_work,
        ),
        (
            "--witness-surface",
            stdio_output(BelvedereCliArtifact::LandscapeDiffWitnessSurface),
            assert_witness_surface,
        ),
        (
            "--provenance-roots",
            stdio_output(BelvedereCliArtifact::LandscapeDiffProvenanceRoots),
            assert_provenance_roots,
        ),
        (
            "--gaps",
            stdio_output(BelvedereCliArtifact::LandscapeDiffGaps),
            assert_gap_items,
        ),
        (
            "--ui-ir",
            stdio_output(BelvedereCliArtifact::LandscapeDiffUiIr),
            assert_landscape_diff_review_ui_ir_projection,
        ),
        (
            "--ui-ir-sections",
            stdio_output(BelvedereCliArtifact::LandscapeDiffUiIrSections),
            assert_landscape_diff_review_ui_ir_sections,
        ),
        (
            "--ui-ir-trace-widgets",
            stdio_output(BelvedereCliArtifact::LandscapeDiffUiIrTraceWidgets),
            assert_landscape_diff_review_ui_ir_trace_widgets,
        ),
        (
            "--ui-ir-viewports",
            stdio_output(BelvedereCliArtifact::LandscapeDiffUiIrViewports),
            assert_ui_ir_viewports,
        ),
        (
            "--ui-ir-minted-standards",
            stdio_output(BelvedereCliArtifact::LandscapeDiffUiIrMintedStandards),
            assert_ui_ir_minted_standards,
        ),
    ];

    for (flag, expected_output, assert_artifact) in cases {
        let args = diff_args_with_empty_previous_assessments(&[flag], &previous_assessments_path)?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err("diff projection artifact alias should be part of the public CLI API")?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);

        let artifact: Value = serde_json::from_slice(&output)
            .into_diagnostic()
            .wrap_err("CLI diff projection artifact alias output should be parseable JSON")?;
        assert_artifact(&artifact);
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_landscape_diff_projection_artifact_aliases_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-projection-artifact-alias-files-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let view_model_path =
        std::env::temp_dir().join(format!("belvedere-diff-view-model-{}.json", process::id()));
    let claim_details_path = std::env::temp_dir().join(format!(
        "belvedere-diff-claim-details-{}.json",
        process::id()
    ));
    let filter_facets_path = std::env::temp_dir().join(format!(
        "belvedere-diff-filter-facets-{}.json",
        process::id()
    ));
    let source_navigation_path = std::env::temp_dir().join(format!(
        "belvedere-diff-source-navigation-{}.json",
        process::id()
    ));
    let proof_chains_path = std::env::temp_dir().join(format!(
        "belvedere-diff-proof-chains-{}.json",
        process::id()
    ));
    let review_workflow_path = std::env::temp_dir().join(format!(
        "belvedere-diff-review-workflow-{}.json",
        process::id()
    ));
    let ui_ir_path =
        std::env::temp_dir().join(format!("belvedere-diff-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 7] = [
        (
            "--view-model",
            view_model_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffViewModel,
                view_model_path.clone(),
            ),
            assert_landscape_diff_review_model,
        ),
        (
            "--claim-details",
            claim_details_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffClaimDetails,
                claim_details_path.clone(),
            ),
            assert_claim_details,
        ),
        (
            "--filter-facets",
            filter_facets_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffFilterFacets,
                filter_facets_path.clone(),
            ),
            assert_filter_facets,
        ),
        (
            "--source-navigation",
            source_navigation_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffSourceNavigation,
                source_navigation_path.clone(),
            ),
            assert_source_navigation_map,
        ),
        (
            "--proof-chains",
            proof_chains_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffProofChains,
                proof_chains_path.clone(),
            ),
            assert_proof_chains,
        ),
        (
            "--review-workflow",
            review_workflow_path.clone(),
            file_output(
                BelvedereCliArtifact::LandscapeDiffReviewWorkflow,
                review_workflow_path.clone(),
            ),
            assert_landscape_diff_review_workflow_queue,
        ),
        (
            "--ui-ir",
            ui_ir_path.clone(),
            file_output(BelvedereCliArtifact::LandscapeDiffUiIr, ui_ir_path.clone()),
            assert_landscape_diff_review_ui_ir_projection,
        ),
    ];

    for (flag, artifact_path, expected_output, assert_artifact) in cases {
        let artifact_path_text = artifact_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary projection artifact path should be UTF-8"))?;
        let args = diff_args_with_empty_previous_assessments(
            &[flag, artifact_path_text],
            &previous_assessments_path,
        )?;
        let command = BelvedereCli::from_args(args.clone())
            .into_diagnostic()
            .wrap_err("diff projection artifact alias file should be part of the public CLI API")?;

        assert_eq!(command.output(), expected_output);

        let mut output = Vec::new();
        let status = run_cli_with_writer(args, &mut output);
        assert_eq!(status, ExitCode::SUCCESS);
        assert!(output.is_empty());

        let artifact_bytes = fs::read(&artifact_path)
            .into_diagnostic()
            .wrap_err("diff projection artifact alias file should be written")?;
        let artifact: Value = serde_json::from_slice(&artifact_bytes)
            .into_diagnostic()
            .wrap_err("persisted diff projection artifact alias should be parseable JSON")?;
        assert_artifact(&artifact);

        fs::remove_file(artifact_path)
            .into_diagnostic()
            .wrap_err("temporary diff projection artifact alias file should be removable")?;
    }

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_ui_ir_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-ui-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--ui-ir-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff UI IR receipt flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffUiIrReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff UI IR receipt output should be parseable JSON")?;
    assert_landscape_diff_review_ui_ir_receipt(&receipt);

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_view_model_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-view-model-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--view-model-receipt",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff view-model receipt flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiffViewModelReceipt)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff view-model receipt output should be parseable JSON")?;
    assert_landscape_diff_review_receipt(&receipt);

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_diff_ui_ir_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-ui-file-empty-assessments-{}.jsonl",
        process::id()
    ));
    let receipt_path = std::env::temp_dir().join(format!(
        "belvedere-diff-ui-ir-receipt-{}.json",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--ui-ir-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary UI IR receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff UI IR receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::LandscapeDiffUiIrReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("diff UI IR receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted diff UI IR receipt should be parseable JSON")?;
    assert_landscape_diff_review_ui_ir_receipt(&receipt);

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary diff UI IR receipt file should be removable")?;
    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_persist_the_landscape_diff_view_model_receipt_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-view-model-file-empty-assessments-{}.jsonl",
        process::id()
    ));
    let receipt_path = std::env::temp_dir().join(format!(
        "belvedere-diff-view-model-receipt-{}.json",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--view-model-receipt",
        "--file",
        receipt_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary view-model receipt path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff view-model receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        file_output(
            BelvedereCliArtifact::LandscapeDiffViewModelReceipt,
            receipt_path.clone()
        )
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output.is_empty());

    let receipt_bytes = fs::read(&receipt_path)
        .into_diagnostic()
        .wrap_err("diff view-model receipt file should be written")?;
    let receipt: Value = serde_json::from_slice(&receipt_bytes)
        .into_diagnostic()
        .wrap_err("persisted diff view-model receipt should be parseable JSON")?;
    assert_landscape_diff_review_receipt(&receipt);

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary diff view-model receipt file should be removable")?;
    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

#[test]
fn cli_can_emit_the_landscape_diff_as_json() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-cli-diff-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let args = [
        "diff",
        "--diff",
        "--stdio",
        "tests/fixtures/registry_dump.json",
        previous_assessments_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary assessment path should be UTF-8"))?,
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("diff JSON flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        stdio_output(BelvedereCliArtifact::LandscapeDiff)
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let diff: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff output should be parseable JSON")?;
    assert_eq!(diff["items"].as_array().map(Vec::len), Some(6));
    assert_eq!(diff["source_record_ids"].as_array().map(Vec::len), Some(7));
    assert!(diff["items"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["change_kind"].as_str() == Some("added")
                && item["review_priority"].as_str() == Some("review")
                && item["review_action"].as_str() == Some("review_new_assessment")
                && item["subject_kind"].as_str() == Some("assessment")
                && item["trace"]["source_record_ids"]
                    .as_array()
                    .is_some_and(|source_record_ids| !source_record_ids.is_empty())
        })
    }));

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}
