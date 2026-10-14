use amenable::{CertRegistry, Certificate, Provenance, RustLanguageProvenance};
use amenable_core::{MetadataRecord, OwnedEntry};
use belvedere::{
    ElicitUiMetadataProjection, MetadataUiIrInput, UI_IR_SECTION_CERTIFICATE_METADATA,
};
use miette::{IntoDiagnostic, Result, WrapErr};

#[test]
fn live_metadata_preserves_order_duplicate_keys_and_values() -> Result<()> {
    belvedere::init_tracing();
    let mut metadata = MetadataRecord::new();
    metadata.push(OwnedEntry::new("motor.max_rpm", 7200_u32));
    metadata.push(OwnedEntry::new("authority", "lab A".to_owned()));
    metadata.push(OwnedEntry::new("authority", "lab B".to_owned()));

    let input = MetadataUiIrInput::from_metadata("source:motor", "Motor", &metadata);
    assert_eq!(input.fields().len(), 3);
    assert_eq!(input.fields()[0].key(), "motor.max_rpm");
    assert_eq!(input.fields()[0].value(), "7200");
    assert_eq!(input.fields()[1].ordinal(), 1);
    assert_eq!(input.fields()[2].value(), "lab B");

    let (projection, receipt) = ElicitUiMetadataProjection
        .project(input)
        .into_diagnostic()
        .wrap_err("live metadata should project")?;
    assert_eq!(receipt.field_count(), 3);
    assert_eq!(receipt.source_id(), "source:motor");
    assert_eq!(
        projection.sections()[0].id(),
        UI_IR_SECTION_CERTIFICATE_METADATA
    );
    assert_eq!(projection.sections()[0].widget_count(), 4);
    assert_eq!(projection.trace_widgets().len(), 4);
    assert_eq!(projection.trace_widgets()[1].label(), "motor.max_rpm: 7200");
    assert_eq!(projection.trace_widgets()[2].label(), "authority: lab A");
    assert_eq!(projection.trace_widgets()[3].label(), "authority: lab B");
    assert!(
        projection
            .trace_widgets()
            .iter()
            .all(|widget| { widget.source_record_ids() == &["source:motor".to_owned()] })
    );
    assert!(projection.tree().nodes().len() >= 4);
    assert_eq!(projection.viewport_verifications().len(), 3);

    let mut changed_metadata = MetadataRecord::new();
    changed_metadata.push(OwnedEntry::new("motor.max_rpm", 7200_u32));
    changed_metadata.push(OwnedEntry::new("authority", "lab A".to_owned()));
    changed_metadata.push(OwnedEntry::new("authority", "lab C".to_owned()));
    let changed = MetadataUiIrInput::from_metadata("source:motor", "Motor", &changed_metadata);
    let (_, changed_receipt) = ElicitUiMetadataProjection
        .project(changed)
        .into_diagnostic()?;
    assert_ne!(receipt.input_digest(), changed_receipt.input_digest());
    assert_ne!(receipt.output_digest(), changed_receipt.output_digest());
    Ok(())
}

#[test]
fn issued_certificate_projects_its_frozen_entries() -> Result<()> {
    belvedere::init_tracing();
    let provenance = RustLanguageProvenance::for_source("amenable", "std::fs");
    let certificate = provenance.certification(&mut CertRegistry::new(), "fs policy");
    let expected = certificate
        .entries()
        .map(|entry| (entry.key().to_owned(), entry.value().to_owned()))
        .collect::<Vec<_>>();

    let input = MetadataUiIrInput::from_certificate("repo:example/certificate:1", &certificate);
    assert_eq!(input.heading(), "Certificate 1 for fs policy");
    assert_eq!(
        input
            .fields()
            .iter()
            .map(|field| (field.key().to_owned(), field.value().to_owned()))
            .collect::<Vec<_>>(),
        expected
    );
    let (projection, receipt) = ElicitUiMetadataProjection
        .project(input)
        .into_diagnostic()
        .wrap_err("issued certificate should project")?;
    assert_eq!(receipt.field_count(), expected.len());
    assert_eq!(projection.trace_widgets().len(), expected.len() + 1);
    for (widget, (key, value)) in projection.trace_widgets()[1..].iter().zip(expected) {
        assert_eq!(widget.label(), &format!("{key}: {value}"));
    }
    Ok(())
}

#[test]
fn empty_metadata_still_has_an_identified_section() -> Result<()> {
    belvedere::init_tracing();
    let input = MetadataUiIrInput::from_metadata("source:empty", "Empty", &MetadataRecord::new());
    let (projection, receipt) = ElicitUiMetadataProjection
        .project(input)
        .into_diagnostic()?;
    assert_eq!(receipt.field_count(), 0);
    assert_eq!(projection.sections()[0].widget_count(), 1);
    assert_eq!(projection.trace_widgets()[0].label(), "Empty");
    Ok(())
}
