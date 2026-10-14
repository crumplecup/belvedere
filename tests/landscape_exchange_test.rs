use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, BelvederePipelineStage, BelvedereStageCertificate,
    LandscapeGapKind, LandscapeInput, LandscapeModeling, ProofExecutionStatus, RawAmenableArtifact,
    Recommendation, RegistryDumpIngestion,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn landscape_modeling_maps_decoded_records_into_a_review_graph() -> Result<()> {
    belvedere::init_tracing();

    let input = fixture_input()?;
    let expected_source_receipt_digests = input.source_receipt_digests().to_vec();
    let (landscape, receipt) = LandscapeModeling
        .model(input)
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
        receipt.item_kind_counts().get("landscape_item_kind_claim"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .item_kind_counts()
            .get("landscape_item_kind_contract"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .item_kind_counts()
            .get("landscape_item_kind_witness"),
        Some(&1)
    );
    assert_eq!(
        receipt.item_kind_counts().get("landscape_item_kind_gap"),
        Some(&4)
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
    assert_landscape_receipt_identity(&receipt);
    assert_eq!(receipt.source_receipt_ids().len(), 2);
    assert!(
        receipt
            .source_receipt_ids()
            .iter()
            .all(|id| id.starts_with("belvedere.receipt.ingestion:"))
    );
    assert_eq!(
        receipt.source_receipt_digests(),
        &expected_source_receipt_digests
    );
    assert!(
        receipt
            .source_receipt_digests()
            .iter()
            .all(|digest| is_hex_digest(digest))
    );
    assert_eq!(receipt.landscape_item_ids().len(), 19);
    assert!(
        receipt
            .landscape_item_ids()
            .contains(&"landscape.claim:belvedere::ingestion::DecodedRegistrySnapshot".to_owned())
    );
    assert!(
        receipt
            .landscape_item_ids()
            .contains(&"landscape.root:belvedere::ingestion::RawAmenableArtifact".to_owned())
    );
    assert!(receipt.landscape_item_ids().contains(
        &"landscape.proof:belvedere::ingestion::DecodedRegistrySnapshot:kani".to_owned()
    ));
    assert!(
        receipt
            .landscape_item_ids()
            .contains(&"landscape.assessment:assess-belvedere-ingestion-1".to_owned())
    );
    assert!(
        receipt.landscape_item_ids().contains(
            &"landscape.gap:belvedere::ingestion::DecodedRegistrySnapshot:missing_assessment"
                .to_owned()
        )
    );

    let certificate = BelvedereStageCertificate::landscape_modeling(&receipt);
    assert_eq!(
        certificate.stage(),
        BelvederePipelineStage::LandscapeModeling
    );
    assert_eq!(
        certificate.counts().get("landscape_items"),
        Some(&receipt.landscape_item_ids().len())
    );
    assert_eq!(
        certificate.counts().get("source_receipts"),
        Some(&receipt.source_receipt_ids().len())
    );
    assert_eq!(
        certificate.counts().get("landscape_item_kind_claim"),
        receipt.item_kind_counts().get("landscape_item_kind_claim")
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .any(|check| check.name() == "landscape_item_ids_classified" && check.passed())
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .any(|check| { check.name() == "landscape_core_counts_match_items" && check.passed() })
    );
    assert!(
        receipt
            .landscape_item_ids()
            .iter()
            .all(|landscape_item_id| certificate.output_ids().contains(landscape_item_id))
    );
    assert_eq!(certificate.input_ids(), receipt.source_receipt_ids());
    assert_eq!(
        certificate.input_digests(),
        receipt.source_receipt_digests()
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
        proof_claim.kani_proofs()[0].status(),
        ProofExecutionStatus::Verified
    );
    assert_eq!(proof_claim.kani_proofs()[0].scope(), "bounded harness");
    assert!(proof_claim.kani_proofs()[0].provides_coverage());
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
fn landscape_modeling_keeps_unverified_backend_runs_out_of_proof_coverage() -> Result<()> {
    belvedere::init_tracing();

    let (landscape, receipt) = LandscapeModeling
        .model(fixture_input_with_registry_and_assessments(
            TIMED_OUT_PROOF_REGISTRY.as_bytes().to_vec(),
            Vec::new(),
        )?)
        .into_diagnostic()
        .wrap_err("landscape modeling should surface unresolved proof execution")?;

    let claim = landscape
        .claims()
        .get("belvedere::proofs::timed_out_case")
        .ok_or_else(|| miette!("timed-out proof record should become a claim"))?;
    assert!(claim.proofs().is_empty());
    assert_eq!(
        claim.kani_proofs()[0].status(),
        ProofExecutionStatus::TimedOut
    );
    assert_eq!(claim.kani_proofs()[0].scope(), "bounded unwinding 8");
    assert!(!claim.kani_proofs()[0].provides_coverage());
    assert!(landscape.gaps().iter().any(|gap| {
        gap.claim_id() == "belvedere::proofs::timed_out_case"
            && gap.kind() == LandscapeGapKind::MissingProof
    }));
    assert!(landscape.gaps().iter().any(|gap| {
        gap.claim_id() == "belvedere::proofs::timed_out_case"
            && gap.kind() == LandscapeGapKind::ProofExecutionUnresolved
            && gap.detail().contains("solver timeout")
    }));
    assert_eq!(
        receipt
            .gap_kind_counts()
            .get("landscape_gap_kind_ProofExecutionUnresolved"),
        Some(&1)
    );
    Ok(())
}

fn assert_landscape_receipt_identity(receipt: &belvedere::LandscapeReceipt) {
    assert!(
        receipt
            .receipt_id()
            .starts_with("belvedere.receipt.landscape:")
    );
    assert_eq!(
        receipt.receipt_id(),
        &format!("belvedere.receipt.landscape:{}", receipt.receipt_digest())
    );
}

fn is_hex_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
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
    fixture_input_with_registry_and_assessments(
        include_bytes!("fixtures/registry_dump.json").to_vec(),
        assessment_bytes,
    )
}

fn fixture_input_with_registry_and_assessments(
    registry_bytes: Vec<u8>,
    assessment_bytes: Vec<u8>,
) -> Result<LandscapeInput> {
    let registry_artifact = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        registry_bytes,
    );
    let assessment_artifact = RawAmenableArtifact::new(
        "tests/fixtures/proof_assessments.jsonl",
        ArtifactKind::ProofAssessments,
        assessment_bytes,
    );

    let (registry, registry_receipt) = RegistryDumpIngestion
        .ingest(registry_artifact)
        .into_diagnostic()
        .wrap_err("registry fixture should satisfy ingestion contract")?;
    let (assessments, assessment_receipt) = AssessmentStreamIngestion
        .ingest(assessment_artifact)
        .into_diagnostic()
        .wrap_err("assessment fixture should satisfy ingestion contract")?;

    Ok(LandscapeInput::from_ingestion(
        registry,
        &registry_receipt,
        assessments,
        &assessment_receipt,
    ))
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

const TIMED_OUT_PROOF_REGISTRY: &str = r#"{
  "evidence_links": [],
  "proof_records": [],
  "contract_records": [],
  "witness_export_records": [],
  "kani_proofs": [
    {
      "id": "belvedere::proofs::timed_out_case",
      "harness": "verify_timed_out_case",
      "package": "belvedere",
      "status": "timed_out",
      "scope": "bounded unwinding 8",
      "failure_reason": "solver timeout"
    }
  ]
}"#;
