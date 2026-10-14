use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, BelvedereContract, BelvedereExchange,
    DecodedRegistrySnapshot, IngestionError, IngestionReceipt, LandscapeInput, LandscapeModeling,
    LandscapeModelingContract, RawAmenableArtifact, RegistryDumpIngestion,
    RegistryDumpIngestionContract,
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
