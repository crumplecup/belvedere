use std::{
    fs,
    path::{Path, PathBuf},
    process::{self, ExitCode},
};

use belvedere::{
    BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_ID, BELVEDERE_ARTIFACT_MANIFEST_SCHEMA_VERSION,
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereCli, BelvedereCliOutput, BelvedereInvocationSurface, BelvederePipelineStage,
    BelvedereRunInput, INPUT_FIDELITY_STANDARD_ID, REPORT_REPRODUCIBILITY_STANDARD_ID,
    UI_IR_SECTION_ASSESSMENT_HISTORY, UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_DETAIL,
    UI_IR_SECTION_FILTERS, UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_REVIEW_WORKFLOW,
    UI_IR_SECTION_SOURCE_NAVIGATION, VIEW_MODEL_FIDELITY_STANDARD_ID, run_belvedere_pipeline,
    run_cli_with_writer,
};
use miette::{IntoDiagnostic, Result, WrapErr};
use serde_json::Value;

type ReceiptAssertion = fn(&Value);

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
    assert_eq!(report.view_model_receipt().counts().filter_facets(), 10);
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
    assert_eq!(report.ui_ir_receipt().counts().section_count(), 13);
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
    assert_eq!(report.manifest().output_artifacts().len(), 9);
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
        report.manifest().run_certificate_id(),
        report.certificate().id()
    );
    assert!(
        report
            .manifest()
            .output_artifacts()
            .iter()
            .any(|artifact| artifact.role() == "ui_ir_projection"
                && artifact.source_stage_role() == "ui_ir_projection"
                && *artifact.item_count() == report.ui_ir_receipt().trace_widget_ids().len())
    );
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
        auditor_stage.input_ids(),
        &vec![report.landscape_receipt().receipt_id().to_owned()]
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
    let ui_stage = report
        .certificate()
        .stage_certificates()
        .iter()
        .find(|certificate| certificate.stage() == BelvederePipelineStage::UiIrProjection)
        .ok_or_else(|| {
            miette::miette!("run certificate should include UI IR projection stage evidence")
        })?;
    assert_eq!(ui_stage.counts().get("ui_ir_sections_populated"), Some(&13));
    assert_eq!(ui_stage.counts().get("ui_ir_sections_empty"), Some(&0));
    assert_eq!(
        ui_stage.counts().get("ui_ir_section_widgets_total"),
        Some(&report.projection().trace_widgets().len())
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_sections_with_source_widgets"),
        Some(&13)
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
    assert!(help.contains("usage: belvedere"));
    assert!(help.contains("belvedere diff"));
    assert!(help.contains("--certificate-json"));
    assert!(help.contains("--certificate-file"));
    assert!(help.contains("--manifest-json"));
    assert!(help.contains("--manifest-file"));
    assert!(help.contains("--run-certificate-json"));
    assert!(help.contains("--run-certificate-file"));
    assert!(help.contains("--diff-json"));
    assert!(help.contains("--diff-receipt-json"));
    assert!(help.contains("--diff-receipt-file"));
    assert!(help.contains("--previous-registry-receipt-json"));
    assert!(help.contains("--previous-registry-receipt-file"));
    assert!(help.contains("--previous-assessment-receipt-json"));
    assert!(help.contains("--previous-assessment-receipt-file"));
    assert!(help.contains("--current-registry-receipt-json"));
    assert!(help.contains("--current-registry-receipt-file"));
    assert!(help.contains("--current-assessment-receipt-json"));
    assert!(help.contains("--current-assessment-receipt-file"));
    assert!(help.contains("--previous-landscape-receipt-json"));
    assert!(help.contains("--previous-landscape-receipt-file"));
    assert!(help.contains("--previous-view-model-json"));
    assert!(help.contains("--previous-view-model-file"));
    assert!(help.contains("--previous-view-model-receipt-json"));
    assert!(help.contains("--previous-view-model-receipt-file"));
    assert!(help.contains("--previous-ui-ir-json"));
    assert!(help.contains("--previous-ui-ir-file"));
    assert!(help.contains("--previous-ui-ir-receipt-json"));
    assert!(help.contains("--previous-ui-ir-receipt-file"));
    assert!(help.contains("--current-landscape-receipt-json"));
    assert!(help.contains("--current-landscape-receipt-file"));
    assert!(help.contains("--current-view-model-receipt-json"));
    assert!(help.contains("--current-view-model-receipt-file"));
    assert!(help.contains("--current-ui-ir-receipt-json"));
    assert!(help.contains("--current-ui-ir-receipt-file"));
    assert!(help.contains("--registry-receipt-json"));
    assert!(help.contains("--registry-receipt-file"));
    assert!(help.contains("--assessment-receipt-json"));
    assert!(help.contains("--assessment-receipt-file"));
    assert!(help.contains("--landscape-receipt-json"));
    assert!(help.contains("--landscape-receipt-file"));
    assert!(help.contains("--view-model-receipt-json"));
    assert!(help.contains("--view-model-receipt-file"));
    assert!(help.contains("--ui-ir-receipt-json"));
    assert!(help.contains("--ui-ir-receipt-file"));
    Ok(())
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
    Ok(["diff"]
        .into_iter()
        .chain(output_args.iter().copied())
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
        receipt["source_record_ids"].as_array().map(Vec::len),
        Some(7)
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
    assert_eq!(receipt["view_item_ids"].as_array().map(Vec::len), Some(60));
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
}

fn assert_ui_ir_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(13));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_HISTORY].as_u64(),
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
    assert_eq!(receipt["output_ids"].as_array().map(Vec::len), Some(153));
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_minted_standard"].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_section"].as_u64(),
        Some(13)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_viewport"].as_u64(),
        Some(3)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_section_root_widget"].as_u64(),
        Some(13)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_trace_widget"].as_u64(),
        Some(60)
    );
    assert_eq!(
        receipt["output_kind_counts"]["ui_ir_output_kind_trace_widget_ref"].as_u64(),
        Some(60)
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
}

fn assert_previous_ui_ir_receipt(receipt: &Value) {
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(13));
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
    assert_eq!(model["filter_facets"].as_array().map(Vec::len), Some(10));
    assert_eq!(model["backend_coverage"].as_array().map(Vec::len), Some(12));
    assert_eq!(model["claim_explorer"].as_array().map(Vec::len), Some(4));
    assert_eq!(model["proof_chains"].as_array().map(Vec::len), Some(4));
    assert_eq!(model["review_workflow"].as_array().map(Vec::len), Some(4));
    assert_eq!(
        model["assessment_history"].as_array().map(Vec::len),
        Some(2)
    );
    assert_eq!(model["source_record_ids"].as_array().map(Vec::len), Some(7));
    assert!(model["claim_explorer"].as_array().is_some_and(|items| {
        items.iter().any(|item| {
            item["claim_id"].as_str() == Some("belvedere::ingestion::DecodedRegistrySnapshot")
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

fn assert_ui_ir_projection(projection: &Value) {
    assert!(projection.get("tree").is_none());
    assert!(projection.get("proofs").is_none());
    assert_eq!(projection["sections"].as_array().map(Vec::len), Some(13));
    assert_eq!(
        projection["trace_widgets"].as_array().map(Vec::len),
        Some(60)
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
    assert!(projection["sections"].as_array().is_some_and(|sections| {
        sections
            .iter()
            .any(|section| section["id"].as_str() == Some(UI_IR_SECTION_ASSESSMENT_HISTORY))
    }));
    assert!(
        projection["minted_standards"]
            .as_array()
            .is_some_and(|standards| standards
                .iter()
                .any(|standard| standard["prop"].as_str() == Some("elicit_ui::WcagVerified")))
    );
}

fn assert_previous_ui_ir_projection(projection: &Value) {
    assert!(projection.get("tree").is_none());
    assert!(projection.get("proofs").is_none());
    assert_eq!(projection["sections"].as_array().map(Vec::len), Some(13));
    assert_eq!(
        projection["trace_widgets"].as_array().map(Vec::len),
        Some(44)
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
    assert_eq!(manifest["counts"]["output_artifacts"].as_u64(), Some(9));
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
    assert!(
        manifest["invariant_checks"]
            .as_array()
            .is_some_and(|checks| checks.iter().any(|check| {
                check["name"].as_str() == Some("artifact_manifest_output_sources_accounted")
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
        Some(9)
    );
    assert!(
        manifest["output_artifacts"]
            .as_array()
            .is_some_and(|artifacts| {
                artifacts.iter().any(|artifact| {
                    artifact["role"].as_str() == Some("ui_ir_projection")
                        && artifact["artifact_kind"].as_str() == Some("elicit_ui_projection")
                        && artifact["source_stage_role"].as_str() == Some("ui_ir_projection")
                })
            })
    );
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
    assert_eq!(manifest["counts"]["stage_certificates"].as_u64(), Some(11));
    assert_eq!(manifest["counts"]["output_artifacts"].as_u64(), Some(20));
    assert_eq!(
        manifest["stage_kind_counts"]["manifest_stage_kind_registry_ingestion"].as_u64(),
        Some(2)
    );
    assert_eq!(
        manifest["stage_kind_counts"]["manifest_stage_kind_landscape_diff"].as_u64(),
        Some(1)
    );
    assert_eq!(
        manifest["output_kind_counts"]["manifest_output_kind_landscape_diff_receipt"].as_u64(),
        Some(1)
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
        Some(11)
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
        manifest["output_artifacts"]
            .as_array()
            .is_some_and(|artifacts| {
                artifacts.iter().any(|artifact| {
                    artifact["role"].as_str() == Some("landscape_diff")
                        && artifact["artifact_kind"].as_str() == Some("landscape_diff")
                        && artifact["item_count"].as_u64() == Some(6)
                })
            })
    );
}

#[test]
fn cli_can_emit_the_run_certificate_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--certificate-json",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("certificate JSON flag should be part of the public CLI API")?;

    assert_eq!(command.output(), BelvedereCliOutput::RunCertificateJson);

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
        Some(3)
    );
    Ok(())
}

#[test]
fn cli_can_emit_the_run_manifest_as_json() -> Result<()> {
    belvedere::init_tracing();

    let args = [
        "--manifest-json",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("manifest JSON flag should be part of the public CLI API")?;

    assert_eq!(command.output(), BelvedereCliOutput::RunManifestJson);

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
        "--registry-receipt-json",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("registry receipt JSON flag should be part of the public CLI API")?;

    assert_eq!(command.output(), BelvedereCliOutput::RunRegistryReceiptJson);

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
        "--assessment-receipt-json",
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
        BelvedereCliOutput::RunAssessmentReceiptJson
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
        "--landscape-receipt-json",
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
        BelvedereCliOutput::RunLandscapeReceiptJson
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
        "--landscape-json",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("landscape graph JSON flag should be part of the public CLI API")?;

    assert_eq!(command.output(), BelvedereCliOutput::RunLandscapeJson);

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
        "--ui-ir-receipt-json",
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("UI IR receipt JSON flag should be part of the public CLI API")?;

    assert_eq!(command.output(), BelvedereCliOutput::RunUiIrReceiptJson);

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
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(13));
    assert_eq!(receipt["section_ids"].as_array().map(Vec::len), Some(13));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_HISTORY].as_u64(),
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
        "--view-model-receipt-json",
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
        BelvedereCliOutput::RunViewModelReceiptJson
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

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--view-model-json",
            BelvedereCliOutput::RunViewModelJson,
            assert_auditor_navigation_model,
        ),
        (
            "--ui-ir-json",
            BelvedereCliOutput::RunUiIrJson,
            assert_ui_ir_projection,
        ),
    ];

    for (flag, expected_output, assert_artifact) in cases {
        let args = [
            flag,
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
        "--registry-receipt-file",
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
        BelvedereCliOutput::RunRegistryReceiptFile(receipt_path.clone())
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
        "--assessment-receipt-file",
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
        BelvedereCliOutput::RunAssessmentReceiptFile(receipt_path.clone())
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
        "--landscape-receipt-file",
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
        BelvedereCliOutput::RunLandscapeReceiptFile(receipt_path.clone())
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
        "--landscape-file",
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
        BelvedereCliOutput::RunLandscapeFile(graph_path.clone())
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
        "--ui-ir-receipt-file",
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
        BelvedereCliOutput::RunUiIrReceiptFile(receipt_path.clone())
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
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(13));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_ASSESSMENT_HISTORY].as_u64(),
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
        "--view-model-receipt-file",
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
        BelvedereCliOutput::RunViewModelReceiptFile(receipt_path.clone())
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
    let ui_ir_path = std::env::temp_dir().join(format!("belvedere-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--view-model-file",
            view_model_path.clone(),
            BelvedereCliOutput::RunViewModelFile(view_model_path.clone()),
            assert_auditor_navigation_model,
        ),
        (
            "--ui-ir-file",
            ui_ir_path.clone(),
            BelvedereCliOutput::RunUiIrFile(ui_ir_path.clone()),
            assert_ui_ir_projection,
        ),
    ];

    for (flag, artifact_path, expected_output, assert_artifact) in cases {
        let artifact_path_text = artifact_path
            .to_str()
            .ok_or_else(|| miette::miette!("temporary projection path should be UTF-8"))?;
        let args = [
            flag,
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
        "--certificate-file",
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
        BelvedereCliOutput::RunCertificateFile(certificate_path.clone())
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
        "--manifest-file",
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
        BelvedereCliOutput::RunManifestFile(manifest_path.clone())
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
        "--certificate-json",
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
        BelvedereCliOutput::LandscapeDiffCertificateJson
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
        "--run-certificate-json",
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
        BelvedereCliOutput::LandscapeDiffRunCertificateJson
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
        Some(11)
    );
    assert_eq!(
        certificate["stage_certificate_digests"]
            .as_array()
            .map(Vec::len),
        Some(11)
    );
    assert_eq!(
        certificate["input_artifact_ids"].as_array().map(Vec::len),
        Some(4)
    );
    assert_eq!(
        certificate["invocation"]["arguments"]
            .as_array()
            .map(Vec::len),
        Some(6)
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
                    stage_certificate["stage"].as_str() == Some("ui_ir_projection")
                        && stage_certificate["counts"]["ui_ir_sections_populated"].as_u64()
                            == Some(13)
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
        "--manifest-json",
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
        BelvedereCliOutput::LandscapeDiffManifestJson
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
        "--run-certificate-file",
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
        BelvedereCliOutput::LandscapeDiffRunCertificateFile(certificate_path.clone())
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
        Some(11)
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
        "--diff-receipt-json",
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
        BelvedereCliOutput::LandscapeDiffReceiptJson
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
        "--diff-receipt-file",
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
        BelvedereCliOutput::LandscapeDiffReceiptFile(receipt_path.clone())
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
            "--previous-registry-receipt-json",
            BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptJson,
            assert_registry_ingestion_receipt,
        ),
        (
            "--previous-assessment-receipt-json",
            BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptJson,
            assert_empty_assessment_ingestion_receipt,
        ),
        (
            "--current-registry-receipt-json",
            BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptJson,
            assert_registry_ingestion_receipt,
        ),
        (
            "--current-assessment-receipt-json",
            BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptJson,
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
        &["--current-registry-receipt-file", receipt_path_text],
        &previous_assessments_path,
    )?;
    let command = BelvedereCli::from_args(args.clone())
        .into_diagnostic()
        .wrap_err("current registry receipt file flag should be part of the public CLI API")?;

    assert_eq!(
        command.output(),
        BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptFile(receipt_path.clone())
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
            "--previous-landscape-receipt-json",
            BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptJson,
            assert_previous_landscape_receipt,
        ),
        (
            "--current-landscape-receipt-json",
            BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptJson,
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
            "--previous-landscape-json",
            BelvedereCliOutput::LandscapeDiffPreviousLandscapeJson,
            assert_previous_landscape_graph,
        ),
        (
            "--current-landscape-json",
            BelvedereCliOutput::LandscapeDiffCurrentLandscapeJson,
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
            "--previous-landscape-receipt-file",
            previous_receipt_path.clone(),
            BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptFile(
                previous_receipt_path.clone(),
            ),
            assert_previous_landscape_receipt,
        ),
        (
            "--current-landscape-receipt-file",
            current_receipt_path.clone(),
            BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptFile(
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
            "--previous-landscape-file",
            previous_graph_path.clone(),
            BelvedereCliOutput::LandscapeDiffPreviousLandscapeFile(previous_graph_path.clone()),
            assert_previous_landscape_graph,
        ),
        (
            "--current-landscape-file",
            current_graph_path.clone(),
            BelvedereCliOutput::LandscapeDiffCurrentLandscapeFile(current_graph_path.clone()),
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
        "--landscape-receipt-json",
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
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptJson
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
        "--landscape-receipt-file",
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
        BelvedereCliOutput::LandscapeDiffLandscapeReceiptFile(receipt_path.clone())
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
        "--landscape-json",
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
        BelvedereCliOutput::LandscapeDiffLandscapeJson
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
        "--landscape-file",
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
        BelvedereCliOutput::LandscapeDiffLandscapeFile(graph_path.clone())
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
            "--previous-view-model-receipt-json",
            BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptJson,
            assert_previous_auditor_navigation_receipt,
        ),
        (
            "--previous-ui-ir-receipt-json",
            BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptJson,
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
            "--previous-view-model-receipt-file",
            view_model_receipt_path.clone(),
            BelvedereCliOutput::LandscapeDiffPreviousViewModelReceiptFile(
                view_model_receipt_path.clone(),
            ),
            assert_previous_auditor_navigation_receipt,
        ),
        (
            "--previous-ui-ir-receipt-file",
            ui_ir_receipt_path.clone(),
            BelvedereCliOutput::LandscapeDiffPreviousUiIrReceiptFile(ui_ir_receipt_path.clone()),
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

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-view-model-json",
            BelvedereCliOutput::LandscapeDiffPreviousViewModelJson,
            assert_previous_auditor_navigation_model,
        ),
        (
            "--previous-ui-ir-json",
            BelvedereCliOutput::LandscapeDiffPreviousUiIrJson,
            assert_previous_ui_ir_projection,
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
    let ui_ir_path =
        std::env::temp_dir().join(format!("belvedere-previous-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--previous-view-model-file",
            view_model_path.clone(),
            BelvedereCliOutput::LandscapeDiffPreviousViewModelFile(view_model_path.clone()),
            assert_previous_auditor_navigation_model,
        ),
        (
            "--previous-ui-ir-file",
            ui_ir_path.clone(),
            BelvedereCliOutput::LandscapeDiffPreviousUiIrFile(ui_ir_path.clone()),
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
            "--current-view-model-receipt-json",
            BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptJson,
            assert_auditor_navigation_receipt,
        ),
        (
            "--current-ui-ir-receipt-json",
            BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptJson,
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
            "--current-view-model-receipt-file",
            view_model_receipt_path.clone(),
            BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptFile(
                view_model_receipt_path.clone(),
            ),
            assert_auditor_navigation_receipt,
        ),
        (
            "--current-ui-ir-receipt-file",
            ui_ir_receipt_path.clone(),
            BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptFile(ui_ir_receipt_path.clone()),
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

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--current-view-model-json",
            BelvedereCliOutput::LandscapeDiffCurrentViewModelJson,
            assert_auditor_navigation_model,
        ),
        (
            "--current-ui-ir-json",
            BelvedereCliOutput::LandscapeDiffCurrentUiIrJson,
            assert_ui_ir_projection,
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
    let ui_ir_path =
        std::env::temp_dir().join(format!("belvedere-current-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--current-view-model-file",
            view_model_path.clone(),
            BelvedereCliOutput::LandscapeDiffCurrentViewModelFile(view_model_path.clone()),
            assert_auditor_navigation_model,
        ),
        (
            "--current-ui-ir-file",
            ui_ir_path.clone(),
            BelvedereCliOutput::LandscapeDiffCurrentUiIrFile(ui_ir_path.clone()),
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

    let cases: [(&str, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--view-model-json",
            BelvedereCliOutput::LandscapeDiffViewModelJson,
            assert_auditor_navigation_model,
        ),
        (
            "--ui-ir-json",
            BelvedereCliOutput::LandscapeDiffUiIrJson,
            assert_ui_ir_projection,
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
    let ui_ir_path =
        std::env::temp_dir().join(format!("belvedere-diff-ui-ir-{}.json", process::id()));
    let cases: [(&str, PathBuf, BelvedereCliOutput, ReceiptAssertion); 2] = [
        (
            "--view-model-file",
            view_model_path.clone(),
            BelvedereCliOutput::LandscapeDiffViewModelFile(view_model_path.clone()),
            assert_auditor_navigation_model,
        ),
        (
            "--ui-ir-file",
            ui_ir_path.clone(),
            BelvedereCliOutput::LandscapeDiffUiIrFile(ui_ir_path.clone()),
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
        "--ui-ir-receipt-json",
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
        BelvedereCliOutput::LandscapeDiffUiIrReceiptJson
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff UI IR receipt output should be parseable JSON")?;
    assert_ui_ir_receipt(&receipt);

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
        "--view-model-receipt-json",
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
        BelvedereCliOutput::LandscapeDiffViewModelReceiptJson
    );

    let mut output = Vec::new();
    let status = run_cli_with_writer(args, &mut output);
    assert_eq!(status, ExitCode::SUCCESS);

    let receipt: Value = serde_json::from_slice(&output)
        .into_diagnostic()
        .wrap_err("CLI diff view-model receipt output should be parseable JSON")?;
    assert_auditor_navigation_receipt(&receipt);

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
        "--ui-ir-receipt-file",
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
        BelvedereCliOutput::LandscapeDiffUiIrReceiptFile(receipt_path.clone())
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
    assert_ui_ir_receipt(&receipt);

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
        "--view-model-receipt-file",
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
        BelvedereCliOutput::LandscapeDiffViewModelReceiptFile(receipt_path.clone())
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
    assert_auditor_navigation_receipt(&receipt);

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
        "--diff-json",
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

    assert_eq!(command.output(), BelvedereCliOutput::LandscapeDiffJson);

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
