use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, AuditorNavigationProjection, ElicitUiProjection,
    LandscapeInput, LandscapeModeling, RawAmenableArtifact, RegistryDumpIngestion,
    UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS, UI_IR_SECTION_OVERVIEW,
    UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_SOURCE_NAVIGATION, UiIrInput, ViewModelInput,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn ui_ir_projection_mints_elicit_ui_standards_and_tree_proofs() -> Result<()> {
    belvedere::init_tracing();

    let (projection, receipt) = ElicitUiProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("fixture view model should project into elicit_ui")?;

    assert_eq!(projection.sections().len(), 9);
    assert!(
        projection
            .sections()
            .iter()
            .any(|section| section.id() == UI_IR_SECTION_OVERVIEW && section.name() == "overview")
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_CLAIM_EXPLORER.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_FILTERS.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_PROOF_CHAINS.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_SOURCE_NAVIGATION.to_owned())
    );
    assert!(
        projection
            .minted_standards()
            .iter()
            .any(|mint| mint.prop() == "elicit_ui::WcagVerified")
    );
    assert!(
        receipt
            .minted_standards()
            .iter()
            .any(|standard| standard.ends_with("ElicitUiIrSourcedStandard"))
    );
    assert!(receipt.counts().node_count() >= projection.trace_widgets().len());

    let _ir_sourced = projection.proofs().ir_sourced();
    let _wcag_verified = projection.proofs().wcag_verified();
    Ok(())
}

#[test]
fn ui_ir_projection_preserves_trace_widgets_for_auditor_deep_dives() -> Result<()> {
    belvedere::init_tracing();

    let (projection, receipt) = ElicitUiProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("fixture view model should project into elicit_ui")?;

    let decoded_claim = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && widget.label().starts_with("Claim ")
        })
        .ok_or_else(|| miette!("claim explorer item should have a traceable widget"))?;
    assert!(
        decoded_claim.source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );

    let gap_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| widget.label().starts_with("Gap "))
        .ok_or_else(|| miette!("gap queue should have traceable widgets"))?;
    assert!(!gap_widget.source_record_ids().is_empty());
    let filter_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| widget.label().starts_with("Filter verifier kani"))
        .ok_or_else(|| miette!("filter facets should have traceable widgets"))?;
    assert_eq!(filter_widget.claim_id(), "belvedere.filter.verifier:kani");
    let chain_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| widget.label().starts_with("Proof chain "))
        .ok_or_else(|| miette!("proof-chain view should have traceable widgets"))?;
    assert!(!chain_widget.source_record_ids().is_empty());
    let source_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget
                .label()
                .starts_with("Source record proof_record:belvedere::ingestion")
        })
        .ok_or_else(|| miette!("source navigation should have traceable widgets"))?;
    assert_eq!(source_widget.source_record_ids().len(), 1);
    let preserved_trace_refs = projection
        .trace_widgets()
        .iter()
        .map(|widget| widget.source_record_ids().len())
        .sum::<usize>();
    assert_eq!(receipt.counts().trace_ref_count(), preserved_trace_refs);
    Ok(())
}

fn fixture_input() -> Result<UiIrInput> {
    let registry_artifact = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        include_bytes!("fixtures/registry_dump.json").to_vec(),
    );
    let assessment_artifact = RawAmenableArtifact::new(
        "tests/fixtures/proof_assessments.jsonl",
        ArtifactKind::ProofAssessments,
        include_bytes!("fixtures/proof_assessments.jsonl").to_vec(),
    );

    let (registry, _registry_receipt) = RegistryDumpIngestion
        .ingest(registry_artifact)
        .into_diagnostic()
        .wrap_err("registry fixture should satisfy ingestion contract")?;
    let (assessments, _assessment_receipt) = AssessmentStreamIngestion
        .ingest(assessment_artifact)
        .into_diagnostic()
        .wrap_err("assessment fixture should satisfy ingestion contract")?;
    let landscape_input = LandscapeInput::new(registry, assessments);
    let (landscape, _landscape_receipt) = LandscapeModeling
        .model(landscape_input)
        .into_diagnostic()
        .wrap_err("landscape modeling is conservative and infallible")?;
    let (model, _view_receipt) = AuditorNavigationProjection
        .project(ViewModelInput::new(landscape))
        .into_diagnostic()
        .wrap_err("view-model projection is trace-preserving and infallible")?;

    Ok(UiIrInput::new(model))
}
