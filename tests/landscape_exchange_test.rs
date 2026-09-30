use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, LandscapeGapKind, LandscapeInput, LandscapeModeling,
    RawAmenableArtifact, Recommendation, RegistryDumpIngestion,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn landscape_modeling_maps_decoded_records_into_a_review_graph() -> Result<()> {
    belvedere::init_tracing();

    let (landscape, receipt) = LandscapeModeling
        .model(fixture_input()?)
        .into_diagnostic()
        .wrap_err("landscape modeling is conservative and infallible")?;

    assert_eq!(receipt.claim_count(), 4);
    assert_eq!(receipt.edge_count(), 1);
    assert_eq!(receipt.proof_count(), 2);
    assert_eq!(receipt.assessment_count(), 2);
    assert_eq!(receipt.root_count(), 3);
    assert!(
        receipt
            .source_record_ids()
            .contains(&"kani_proof:belvedere::ingestion::record_identity_is_preserved".to_owned())
    );

    let decoded_claim = landscape
        .claims()
        .get("belvedere::ingestion::DecodedRegistrySnapshot")
        .ok_or_else(|| miette!("registry evidence should become a claim"))?;
    assert!(
        decoded_claim
            .bases()
            .contains("belvedere::ingestion::RawAmenableArtifact")
    );
    assert!(decoded_claim.proofs().contains("kani"));
    assert_eq!(decoded_claim.contracts()[0].kind(), "ensures");
    assert_eq!(decoded_claim.witness_surfaces()[0].checked(), 1);

    let proof_claim = landscape
        .claims()
        .get("belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| {
            miette!("Kani proof registration should become an addressable proof claim")
        })?;
    assert!(proof_claim.proofs().contains("kani"));
    assert_eq!(proof_claim.kani_proofs()[0].package(), "belvedere");
    assert_eq!(
        proof_claim.assessment_ids(),
        &["assess-belvedere-ingestion-1".to_owned()]
    );

    assert!(
        landscape
            .root_claim_ids()
            .contains(&"belvedere::ingestion::RawAmenableArtifact")
    );
    assert_eq!(
        landscape.assessments()[0].recommendation(),
        Recommendation::Strengthen
    );
    Ok(())
}

#[test]
fn landscape_modeling_surfaces_conservative_gaps() -> Result<()> {
    belvedere::init_tracing();

    let (landscape, _receipt) = LandscapeModeling
        .model(fixture_input()?)
        .into_diagnostic()
        .wrap_err("landscape modeling is conservative and infallible")?;

    assert!(landscape.gaps().iter().any(|gap| {
        gap.claim_id() == "belvedere::ingestion::RawAmenableArtifact"
            && gap.kind() == LandscapeGapKind::MissingProof
    }));
    assert!(landscape.gaps().iter().any(|gap| {
        gap.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
            && gap.kind() == LandscapeGapKind::MissingAssessment
    }));
    assert!(landscape.gaps().iter().any(|gap| {
        gap.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
            && gap.kind() == LandscapeGapKind::TrustedOrOpaqueWitnessSupport
    }));
    Ok(())
}

fn fixture_input() -> Result<LandscapeInput> {
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

    Ok(LandscapeInput::new(registry, assessments))
}
