use belvedere::{BelvedereRunInput, run_belvedere_pipeline};
use miette::{IntoDiagnostic, Result, WrapErr};

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
    assert_eq!(report.ui_ir_receipt().counts().section_count(), 6);
    assert_eq!(report.certificate().stage_certificates().len(), 5);
    assert_eq!(report.certificate().input_artifact_ids().len(), 2);
    assert_eq!(report.certificate().stage_certificate_ids().len(), 5);
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
        first.certificate().stage_certificate_ids(),
        second.certificate().stage_certificate_ids()
    );

    let encoded = serde_json::to_string(first.certificate())
        .into_diagnostic()
        .wrap_err("run certificate should serialize as portable JSON evidence")?;
    assert!(encoded.contains("belvedere.run:"));
    assert!(encoded.contains("ui_ir_projection"));
    Ok(())
}
