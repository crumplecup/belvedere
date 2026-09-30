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
    assert!(report.projection().trace_widgets().len() >= 4);
    Ok(())
}
