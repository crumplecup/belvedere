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
    assert_eq!(
        receipt
            .assessment_recency_counts()
            .get("assessment_recency_current"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .gap_kind_counts()
            .get("landscape_gap_kind_MissingProof"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .gap_kind_counts()
            .get("landscape_gap_kind_MissingAssessment"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .gap_kind_counts()
            .get("landscape_gap_kind_TrustedOrOpaqueWitnessSupport"),
        Some(&1)
    );
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

#[test]
fn landscape_modeling_marks_superseded_assessments() -> Result<()> {
    belvedere::init_tracing();

    let (landscape, receipt) = LandscapeModeling
        .model(fixture_input_with_assessments(
            STALE_ASSESSMENTS.as_bytes().to_vec(),
        )?)
        .into_diagnostic()
        .wrap_err("landscape modeling should distinguish assessment recency")?;
    assert_eq!(
        receipt
            .assessment_recency_counts()
            .get("assessment_recency_current"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .assessment_recency_counts()
            .get("assessment_recency_superseded"),
        Some(&1)
    );

    let older = landscape
        .assessments()
        .iter()
        .find(|assessment| assessment.id() == "assess-old")
        .ok_or_else(|| miette!("older assessment should be modeled"))?;
    assert!(!older.current());
    assert_eq!(older.superseded_by(), Some("assess-current"));

    let current = landscape
        .assessments()
        .iter()
        .find(|assessment| assessment.id() == "assess-current")
        .ok_or_else(|| miette!("current assessment should be modeled"))?;
    assert!(current.current());
    assert_eq!(current.superseded_by(), None);
    Ok(())
}

fn fixture_input() -> Result<LandscapeInput> {
    fixture_input_with_assessments(include_bytes!("fixtures/proof_assessments.jsonl").to_vec())
}

fn fixture_input_with_assessments(assessment_bytes: Vec<u8>) -> Result<LandscapeInput> {
    let registry_artifact = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        include_bytes!("fixtures/registry_dump.json").to_vec(),
    );
    let assessment_artifact = RawAmenableArtifact::new(
        "tests/fixtures/proof_assessments.jsonl",
        ArtifactKind::ProofAssessments,
        assessment_bytes,
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

const STALE_ASSESSMENTS: &str = concat!(
    r#"{"version":"0.1.0","assessment_id":"assess-old","proof_id":"belvedere::ingestion::record_identity_is_preserved","reviewer":"belvedere-maintainers","timestamp":1790550000,"#,
    r#""rubric":{"claim_alignment":4,"assumption_adequacy":4,"model_fidelity":4,"assertion_strength":2,"adversarial_coverage":3,"clarity":4},"#,
    r#""recommendation":"strengthen","resolution_path":"strengthen_current_proof","comment":"Older review remains visible but is no longer current."}"#,
    "\n",
    r#"{"version":"0.1.0","assessment_id":"assess-current","proof_id":"belvedere::ingestion::record_identity_is_preserved","reviewer":"belvedere-maintainers","timestamp":1790550060,"#,
    r#""rubric":{"claim_alignment":4,"assumption_adequacy":4,"model_fidelity":4,"assertion_strength":4,"adversarial_coverage":4,"clarity":4},"#,
    r#""recommendation":"accept","resolution_path":"keep_current_proof","comment":"Latest review supersedes the earlier strengthening request."}"#,
    "\n",
);
