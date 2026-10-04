use std::{
    fs,
    process::{self, ExitCode},
};

use belvedere::{
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereCli, BelvedereCliOutput, BelvedereInvocationSurface, BelvederePipelineStage,
    BelvedereRunInput, INPUT_FIDELITY_STANDARD_ID, UI_IR_SECTION_BACKEND_COVERAGE,
    UI_IR_SECTION_FILTERS, UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_REVIEW_WORKFLOW,
    UI_IR_SECTION_SOURCE_NAVIGATION, run_belvedere_pipeline, run_cli_with_writer,
};
use miette::{IntoDiagnostic, Result, WrapErr};
use serde_json::Value;

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
    assert_eq!(report.ui_ir_receipt().counts().section_count(), 11);
    assert!(
        report
            .ui_ir_receipt()
            .section_ids()
            .contains(&UI_IR_SECTION_PROOF_CHAINS.to_owned())
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
    assert_eq!(report.certificate().stage_certificate_ids().len(), 5);
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
                .get("view_model_trace_unique_source_records")
                == Some(&7))
    );
    assert!(
        report
            .certificate()
            .stage_certificates()
            .iter()
            .any(|certificate| certificate.counts().get("role_button") == Some(&21))
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
    let ui_stage = report
        .certificate()
        .stage_certificates()
        .iter()
        .find(|certificate| certificate.stage() == BelvederePipelineStage::UiIrProjection)
        .ok_or_else(|| {
            miette::miette!("run certificate should include UI IR projection stage evidence")
        })?;
    assert_eq!(ui_stage.counts().get("ui_ir_sections_populated"), Some(&11));
    assert_eq!(ui_stage.counts().get("ui_ir_sections_empty"), Some(&0));
    assert_eq!(
        ui_stage.counts().get("ui_ir_section_widgets_total"),
        Some(&report.projection().trace_widgets().len())
    );
    assert_eq!(
        ui_stage.counts().get("ui_ir_sections_with_source_widgets"),
        Some(&11)
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
    assert!(help.contains("--run-certificate-json"));
    assert!(help.contains("--run-certificate-file"));
    assert!(help.contains("--diff-json"));
    assert!(help.contains("--ui-ir-receipt-json"));
    assert!(help.contains("--ui-ir-receipt-file"));
    Ok(())
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
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(11));
    assert_eq!(receipt["section_ids"].as_array().map(Vec::len), Some(11));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );
    assert_eq!(receipt["role_counts"]["role_button"].as_u64(), Some(21));
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
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(11));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary UI IR receipt file should be removable")?;
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
        Some(9)
    );
    assert_eq!(
        certificate["stage_certificate_digests"]
            .as_array()
            .map(Vec::len),
        Some(9)
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
                            == Some(11)
                })
            )
    );

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
        Some(9)
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
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_eq!(receipt["counts"]["section_count"].as_u64(), Some(11));
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );
    assert_eq!(
        receipt["trace_source_counts"]["trace_unique_source_records"].as_u64(),
        Some(7)
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
    assert_eq!(
        receipt["contract"].as_str(),
        Some("belvedere.ui_ir.elicit_ui_projection")
    );
    assert_eq!(
        receipt["trace_source_counts"]["trace_unique_source_records"].as_u64(),
        Some(7)
    );
    assert_eq!(
        receipt["section_widget_counts"][UI_IR_SECTION_REVIEW_WORKFLOW].as_u64(),
        Some(4)
    );

    fs::remove_file(&receipt_path)
        .into_diagnostic()
        .wrap_err("temporary diff UI IR receipt file should be removable")?;
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
