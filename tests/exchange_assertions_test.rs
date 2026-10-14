use belvedere::{
    ArtifactKind, BelvedereContract, BelvedereExchange, DecodedRegistrySnapshot, IngestionError,
    IngestionReceipt, RawAmenableArtifact, RegistryDumpIngestion, RegistryDumpIngestionContract,
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
