use std::{
    fs,
    process::{self, ExitCode},
};

use belvedere::{
    BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID, BELVEDERE_RUN_CERTIFICATE_SCHEMA_VERSION,
    BELVEDERE_STAGE_CERTIFICATE_SCHEMA_ID, BELVEDERE_STAGE_CERTIFICATE_SCHEMA_VERSION,
    BelvedereCli, BelvedereCliOutput, BelvedereInvocationSurface, BelvedereRunInput,
    UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_FILTERS, UI_IR_SECTION_PROOF_CHAINS,
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
    assert_eq!(report.view_model_receipt().counts().filter_facets(), 9);
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
    assert_eq!(report.view_model_receipt().counts().source_items(), 7);
    assert_eq!(report.ui_ir_receipt().counts().section_count(), 10);
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
        first.certificate().stage_certificate_ids(),
        second.certificate().stage_certificate_ids()
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
    assert!(help.contains("--certificate-json"));
    assert!(help.contains("--certificate-file"));
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
        certificate["input_artifact_ids"].as_array().map(Vec::len),
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
    assert!(
        certificate["stage_certificates"]
            .as_array()
            .is_some_and(
                |stage_certificates| stage_certificates.iter().all(|stage_certificate| {
                    stage_certificate["invariant_checks"]
                        .as_array()
                        .is_some_and(|checks| !checks.is_empty())
                })
            )
    );

    fs::remove_file(&certificate_path)
        .into_diagnostic()
        .wrap_err("temporary certificate file should be removable")?;
    Ok(())
}
