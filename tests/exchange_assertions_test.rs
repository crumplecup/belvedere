use belvedere::{
    AcceptedAmenableArtifactFormat, ArtifactKind, AssessmentStreamIngestion, BelvedereContract,
    BelvedereExchange, BelvedereInvariantCheck, BelvedereStageCertificate,
    BelvedereTransitionReceipt, DecodedRegistrySnapshot, IngestionError, IngestionReceipt,
    LandscapeInput, LandscapeModeling, LandscapeModelingContract, RawAmenableArtifact,
    RegistryDumpIngestion, RegistryDumpIngestionContract,
};
use miette::{IntoDiagnostic, Result, miette};

struct ForgedRegistryExchange {
    output: DecodedRegistrySnapshot,
    receipt: IngestionReceipt,
}

impl BelvedereExchange<RegistryDumpIngestionContract> for ForgedRegistryExchange {
    type Error = IngestionError;

    fn transform(
        &self,
        input: &RawAmenableArtifact,
    ) -> std::result::Result<(DecodedRegistrySnapshot, IngestionReceipt), Self::Error> {
        assert_eq!(input.kind(), ArtifactKind::RegistryDump);
        Ok((self.output.clone(), self.receipt.clone()))
    }
}

fn fixture_artifact() -> RawAmenableArtifact {
    RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        include_bytes!("fixtures/registry_dump.json").to_vec(),
    )
}

fn unverified_registry_receipt() -> Result<IngestionReceipt> {
    let artifact = fixture_artifact();
    let snapshot: DecodedRegistrySnapshot =
        serde_json::from_slice(artifact.bytes()).into_diagnostic()?;
    Ok(IngestionReceipt::new(
        RegistryDumpIngestionContract::NAME,
        &artifact,
        AcceptedAmenableArtifactFormat::RegistryDumpJson,
        snapshot.decoded_counts(),
        snapshot.unsupported_fields(),
        Vec::new(),
        snapshot.source_record_ids(),
    ))
}

#[test]
fn stage_certificate_requires_complete_passing_transition_assertions() -> Result<()> {
    belvedere::init_tracing();
    let raw = unverified_registry_receipt()?;
    let missing = BelvedereStageCertificate::registry_ingestion(&raw)
        .err()
        .ok_or_else(|| miette!("unverified receipt should not certify"))?;
    assert!(
        missing
            .to_string()
            .contains("transition_assertions_present")
    );

    let mut failed = raw.clone();
    let false_check = BelvedereInvariantCheck::new(
        "registry_input_kind".to_owned(),
        false,
        "input kind differs".to_owned(),
    );
    assert!(failed.record_transition_checks(vec![false_check]).is_err());

    let mut partial = raw;
    partial
        .record_transition_checks(vec![BelvedereInvariantCheck::new(
            "registry_input_kind".to_owned(),
            true,
            "kind matched".to_owned(),
        )])
        .into_diagnostic()?;
    let incomplete = BelvedereStageCertificate::registry_ingestion(&partial)
        .err()
        .ok_or_else(|| miette!("partial assertion set should not certify"))?;
    assert!(
        incomplete
            .to_string()
            .contains("transition_assertion_set_complete")
    );
    Ok(())
}

#[test]
fn genuine_stage_certificate_carries_checked_transition_assertions() -> Result<()> {
    belvedere::init_tracing();
    let (_, receipt) = RegistryDumpIngestion
        .ingest(fixture_artifact())
        .into_diagnostic()?;
    let certificate = BelvedereStageCertificate::registry_ingestion(&receipt).into_diagnostic()?;
    let names = receipt
        .transition_checks()
        .iter()
        .map(|check| check.name().as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, RegistryDumpIngestionContract::ASSERTION_NAMES);
    assert!(names.iter().all(|name| {
        certificate
            .invariant_checks()
            .iter()
            .any(|check| check.name() == name && check.passed())
    }));
    Ok(())
}

#[test]
fn identical_registry_occurrences_cannot_share_a_certified_source_id() -> Result<()> {
    belvedere::init_tracing();
    let mut registry: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    let records = registry["evidence_links"]
        .as_array_mut()
        .ok_or_else(|| miette!("evidence links should be an array"))?;
    records.push(records[0].clone());
    let input = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        serde_json::to_vec(&registry).into_diagnostic()?,
    );
    let error = RegistryDumpIngestion
        .ingest(input)
        .err()
        .ok_or_else(|| miette!("duplicate source identities should fail ingestion"))?;
    assert!(
        error
            .to_string()
            .contains("registry_source_occurrences_distinguishable")
    );
    Ok(())
}

#[test]
fn identical_assessment_occurrences_cannot_share_a_certified_source_id() -> Result<()> {
    belvedere::init_tracing();
    let source = include_str!("fixtures/proof_assessments.jsonl");
    let first = source
        .lines()
        .next()
        .ok_or_else(|| miette!("assessment fixture should have a line"))?;
    let duplicated = format!("{first}\n{first}\n");
    let input = RawAmenableArtifact::new(
        "tests/fixtures/proof_assessments.jsonl",
        ArtifactKind::ProofAssessments,
        duplicated.into_bytes(),
    );
    let error = AssessmentStreamIngestion
        .ingest(input)
        .err()
        .ok_or_else(|| miette!("duplicate assessment identities should fail ingestion"))?;
    assert!(
        error
            .to_string()
            .contains("assessment_source_occurrences_distinguishable")
    );
    Ok(())
}

fn landscape_error_for_registry_mutation(mutated: serde_json::Value) -> Result<String> {
    let (original, registry_receipt) = RegistryDumpIngestion
        .ingest(fixture_artifact())
        .into_diagnostic()?;
    let (assessments, assessment_receipt) = AssessmentStreamIngestion
        .ingest(RawAmenableArtifact::new(
            "tests/fixtures/proof_assessments.jsonl",
            ArtifactKind::ProofAssessments,
            include_bytes!("fixtures/proof_assessments.jsonl").to_vec(),
        ))
        .into_diagnostic()?;
    let altered: DecodedRegistrySnapshot = serde_json::from_value(mutated).into_diagnostic()?;
    let expected_input = LandscapeInput::from_ingestion(
        original,
        &registry_receipt,
        assessments.clone(),
        &assessment_receipt,
    );
    let altered_input = LandscapeInput::from_ingestion(
        altered,
        &registry_receipt,
        assessments,
        &assessment_receipt,
    );
    let (altered_output, receipt) = LandscapeModeling.model(altered_input).into_diagnostic()?;
    let error =
        LandscapeModelingContract::verify_transition(&expected_input, &altered_output, &receipt)
            .err()
            .ok_or_else(|| miette!("mutated landscape should fail the contract"))?;
    Ok(error.to_string())
}

#[test]
fn landscape_rejects_invented_claim() -> Result<()> {
    belvedere::init_tracing();
    let mut registry: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    registry["evidence_links"][0]["name"] = "invented::claim".into();
    let error = landscape_error_for_registry_mutation(registry)?;
    assert!(error.contains("landscape_claim_payloads_preserved"));
    Ok(())
}

#[test]
fn landscape_rejects_replaced_source_link_despite_equal_counts() -> Result<()> {
    belvedere::init_tracing();
    let mut registry: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    registry["evidence_links"][0]["index"] = 1.into();
    let error = landscape_error_for_registry_mutation(registry)?;
    assert!(error.contains("landscape_edges_complete"));
    Ok(())
}

#[test]
fn exchange_rejects_altered_field_with_preserved_record_identity() -> Result<()> {
    belvedere::init_tracing();
    let input = fixture_artifact();
    let (_, receipt) = RegistryDumpIngestion
        .ingest(input.clone())
        .into_diagnostic()?;
    let mut altered: serde_json::Value = serde_json::from_slice(input.bytes()).into_diagnostic()?;
    altered["contract_records"][0]["fragment"] = "altered obligation".into();
    let output = serde_json::from_value(altered).into_diagnostic()?;
    let forged = ForgedRegistryExchange { output, receipt };

    let error = forged
        .exchange(input)
        .err()
        .ok_or_else(|| miette!("altered field should fail the exchange"))?;
    assert!(error.to_string().contains("ingestion failed"));
    Ok(())
}

#[test]
fn exchange_rejects_receipt_from_another_input() -> Result<()> {
    belvedere::init_tracing();
    let input = fixture_artifact();
    let (output, _) = RegistryDumpIngestion
        .ingest(input.clone())
        .into_diagnostic()?;
    let other = RawAmenableArtifact::new(
        "other/registry_dump.json",
        ArtifactKind::RegistryDump,
        input.bytes().clone(),
    );
    let (_, receipt) = RegistryDumpIngestion.ingest(other).into_diagnostic()?;
    let forged = ForgedRegistryExchange { output, receipt };

    assert!(forged.exchange(input).is_err());
    Ok(())
}

#[test]
fn direct_contract_verification_accepts_genuine_exchange() -> Result<()> {
    belvedere::init_tracing();
    let input = fixture_artifact();
    let (output, receipt) = RegistryDumpIngestion
        .ingest(input.clone())
        .into_diagnostic()?;
    RegistryDumpIngestionContract::verify_transition(&input, &output, &receipt)
        .into_diagnostic()?;
    Ok(())
}

#[test]
fn landscape_rejects_dropped_witness_child_with_unchanged_counts() -> Result<()> {
    belvedere::init_tracing();
    let original: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    let mut truncated = original.clone();
    let witness = truncated["witness_export_records"][0]["artifact"]
        .as_object_mut()
        .ok_or_else(|| miette!("witness artifact should be an object"))?;
    witness.remove("kind");

    let altered_artifact = RawAmenableArtifact::new(
        "tests/fixtures/registry_dump.json",
        ArtifactKind::RegistryDump,
        serde_json::to_vec(&truncated).into_diagnostic()?,
    );
    let (altered_registry, registry_receipt) = RegistryDumpIngestion
        .ingest(altered_artifact)
        .into_diagnostic()?;
    let (assessments, assessment_receipt) = AssessmentStreamIngestion
        .ingest(RawAmenableArtifact::new(
            "tests/fixtures/proof_assessments.jsonl",
            ArtifactKind::ProofAssessments,
            include_bytes!("fixtures/proof_assessments.jsonl").to_vec(),
        ))
        .into_diagnostic()?;
    let intact_registry: DecodedRegistrySnapshot =
        serde_json::from_value(original).into_diagnostic()?;
    let altered_input = LandscapeInput::from_ingestion(
        altered_registry,
        &registry_receipt,
        assessments.clone(),
        &assessment_receipt,
    );
    let intact_input = LandscapeInput::from_ingestion(
        intact_registry,
        &registry_receipt,
        assessments,
        &assessment_receipt,
    );
    let (altered_output, receipt) = LandscapeModeling.model(altered_input).into_diagnostic()?;

    let error =
        LandscapeModelingContract::verify_transition(&intact_input, &altered_output, &receipt)
            .err()
            .ok_or_else(|| miette!("missing witness child should fail landscape verification"))?;
    assert!(
        error
            .to_string()
            .contains("landscape_claim_payloads_preserved")
    );
    Ok(())
}
