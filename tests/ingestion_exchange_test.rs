use belvedere::{
    AcceptedAmenableArtifactFormat, ArtifactKind, AssessmentStreamIngestion,
    BelvedereStageCertificate, INPUT_FIDELITY_STANDARD_ID, ProofExecutionStatus,
    RawAmenableArtifact, Recommendation, RegistryDumpIngestion, ResolutionPath,
    SCHEMA_HONESTY_STANDARD_ID,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn registry_ingestion_distinguishes_kani_registrations_for_one_proof() -> Result<()> {
    belvedere::init_tracing();

    let mut registry: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    let proofs = registry["kani_proofs"]
        .as_array_mut()
        .ok_or_else(|| miette!("Kani proof fixture should be an array"))?;
    let mut second = proofs[0].clone();
    second["harness"] = "verify_secondary".into();
    proofs.push(second);
    let artifact = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        serde_json::to_vec(&registry).into_diagnostic()?,
    );

    let (snapshot, receipt) = RegistryDumpIngestion.ingest(artifact).into_diagnostic()?;
    let ids = snapshot.source_record_ids();
    assert_eq!(snapshot.kani_proofs().len(), 2);
    assert!(ids.contains(&"kani_proof:belvedere::ingestion::record_identity_is_preserved@belvedere:verify_record_identity_is_preserved".to_owned()));
    assert!(ids.contains(&"kani_proof:belvedere::ingestion::record_identity_is_preserved@belvedere:verify_secondary".to_owned()));
    assert_eq!(receipt.source_record_ids(), &ids);
    Ok(())
}

#[test]
fn registry_ingestion_preserves_record_identity_and_unknown_fields() -> Result<()> {
    belvedere::init_tracing();

    let artifact = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        include_bytes!("fixtures/registry_dump.json").to_vec(),
    );

    let (snapshot, receipt) = RegistryDumpIngestion
        .ingest(artifact)
        .into_diagnostic()
        .wrap_err("registry fixture should satisfy ingestion contract")?;

    assert_eq!(snapshot.evidence_links().len(), 1);
    assert_eq!(
        snapshot.evidence_links()[0].name(),
        "belvedere::ingestion::DecodedRegistrySnapshot"
    );
    assert_eq!(snapshot.proof_records()[0].verifier(), "kani");
    assert_eq!(snapshot.contract_records()[0].kind(), "ensures");
    assert_eq!(snapshot.witness_export_records()[0].checked(), 1);
    assert_eq!(
        snapshot.kani_proofs()[0].id(),
        "belvedere::ingestion::record_identity_is_preserved"
    );
    assert_eq!(
        snapshot.kani_proofs()[0].status(),
        ProofExecutionStatus::Verified
    );
    assert_eq!(snapshot.kani_proofs()[0].scope(), Some("bounded harness"));
    assert_eq!(snapshot.kani_proofs()[0].failure_reason(), None);

    assert_eq!(
        receipt.format(),
        AcceptedAmenableArtifactFormat::RegistryDumpJson
    );
    assert_ingestion_receipt_identity(&receipt);
    assert_eq!(receipt.decoded_counts()["evidence_links"], 1);
    assert_eq!(
        receipt
            .source_record_kind_counts()
            .get("source_record_evidence_link"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .source_record_kind_counts()
            .get("source_record_proof_record"),
        Some(&1)
    );
    assert!(
        receipt.source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );
    assert!(
        receipt
            .unsupported_fields()
            .iter()
            .any(|field| field.record() == "registry" && field.field() == "next_schema_hint")
    );
    assert_eq!(
        receipt
            .unsupported_field_record_counts()
            .get("unsupported_field_record_registry"),
        Some(&1)
    );
    assert!(
        receipt.unsupported_fields().iter().any(|field| {
            field.record() == "evidence_links[0]" && field.field() == "future_field"
        })
    );
    assert_eq!(
        receipt
            .unsupported_field_record_counts()
            .get("unsupported_field_record_evidence_links"),
        Some(&1)
    );

    let certificate = BelvedereStageCertificate::registry_ingestion(&receipt);
    assert_eq!(
        certificate.input_digests(),
        &vec![receipt.digest().to_owned()]
    );
    assert!(
        certificate
            .standards()
            .contains(&INPUT_FIDELITY_STANDARD_ID.to_owned())
    );
    assert!(
        certificate
            .standards()
            .contains(&SCHEMA_HONESTY_STANDARD_ID.to_owned())
    );
    assert_eq!(
        certificate.counts().get("source_record_kani_proof"),
        Some(&1)
    );
    assert_eq!(
        certificate
            .counts()
            .get("unsupported_field_record_evidence_links"),
        Some(&1)
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .any(
                |check| check.name() == "ingestion_source_record_ids_classified" && check.passed()
            )
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .any(
                |check| check.name() == "ingestion_decoded_counts_match_source_records"
                    && check.passed()
            )
    );
    Ok(())
}

#[test]
fn assessment_ingestion_preserves_current_and_legacy_shapes() -> Result<()> {
    belvedere::init_tracing();

    let artifact = RawAmenableArtifact::new(
        "tests/fixtures/proof_assessments.jsonl",
        ArtifactKind::ProofAssessments,
        include_bytes!("fixtures/proof_assessments.jsonl").to_vec(),
    );

    let (stream, receipt) = AssessmentStreamIngestion
        .ingest(artifact)
        .into_diagnostic()
        .wrap_err("assessment fixture should satisfy ingestion contract")?;

    assert_eq!(stream.assessments().len(), 2);

    let current = &stream.assessments()[0];
    assert_eq!(
        current.assessment_id(),
        Some("assess-belvedere-ingestion-1")
    );
    assert_eq!(current.timestamp(), 1_790_550_000);
    assert_eq!(current.recommendation(), Recommendation::Strengthen);
    assert_eq!(
        current.resolution_path(),
        Some(ResolutionPath::StrengthenCurrentProof)
    );
    assert_eq!(current.source_line(), 1);

    let legacy = &stream.assessments()[1];
    assert_eq!(legacy.schema_version(), Some(1));
    assert_eq!(legacy.timestamp(), 1_790_550_060);
    assert_eq!(legacy.recommendation(), Recommendation::Accept);
    assert_eq!(legacy.source_line(), 2);

    assert_eq!(receipt.decoded_counts()["assessments"], 2);
    assert_ingestion_receipt_identity(&receipt);
    assert_eq!(
        receipt
            .source_record_kind_counts()
            .get("source_record_assessment"),
        Some(&2)
    );
    assert!(
        receipt
            .source_record_ids()
            .contains(&"assessment:assess-belvedere-ingestion-1".to_owned())
    );
    assert!(receipt.unsupported_fields().iter().any(|field| {
        field.record() == "assessment_line[1]" && field.field() == "future_assessment_field"
    }));
    assert_eq!(
        receipt
            .unsupported_field_record_counts()
            .get("unsupported_field_record_assessment_line"),
        Some(&1)
    );

    let certificate = BelvedereStageCertificate::assessment_ingestion(&receipt);
    assert_eq!(
        certificate.input_digests(),
        &vec![receipt.digest().to_owned()]
    );
    assert!(
        certificate
            .standards()
            .contains(&INPUT_FIDELITY_STANDARD_ID.to_owned())
    );
    assert!(
        certificate
            .standards()
            .contains(&SCHEMA_HONESTY_STANDARD_ID.to_owned())
    );
    assert_eq!(
        certificate.counts().get("source_record_assessment"),
        Some(&2)
    );
    assert_eq!(
        certificate
            .counts()
            .get("unsupported_field_record_assessment_line"),
        Some(&1)
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .any(
                |check| check.name() == "ingestion_source_record_ids_classified" && check.passed()
            )
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .any(
                |check| check.name() == "ingestion_decoded_counts_match_source_records"
                    && check.passed()
            )
    );
    Ok(())
}

fn assert_ingestion_receipt_identity(receipt: &belvedere::IngestionReceipt) {
    assert!(
        receipt
            .receipt_id()
            .starts_with("belvedere.receipt.ingestion:")
    );
    assert_eq!(
        receipt.receipt_id(),
        &format!("belvedere.receipt.ingestion:{}", receipt.receipt_digest())
    );
}

#[test]
fn registry_ingestion_rejects_the_wrong_artifact_kind() -> Result<()> {
    belvedere::init_tracing();

    let artifact = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::ProofAssessments,
        include_bytes!("fixtures/registry_dump.json").to_vec(),
    );

    let error = RegistryDumpIngestion
        .ingest(artifact)
        .err()
        .ok_or_else(|| miette!("wrong artifact kind should fail the registry exchange"))?;
    let wrong_kind = error
        .wrong_artifact_kind_source()
        .ok_or_else(|| miette!("ingestion error should describe an artifact-kind mismatch"))?;

    assert_eq!(wrong_kind.expected(), ArtifactKind::RegistryDump);
    assert_eq!(wrong_kind.actual(), ArtifactKind::ProofAssessments);
    Ok(())
}
