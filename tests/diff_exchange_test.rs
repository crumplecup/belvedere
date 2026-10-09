use std::{collections::BTreeMap, fs, process};

use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID,
    BelvedereLandscapeDiffRunInput, BelvederePipelineStage, BelvedereRunInput,
    BelvedereStageCertificate, LANDSCAPE_CONSTRUCTION_STANDARD_ID, LandscapeDiffChangeKind,
    LandscapeDiffInput, LandscapeDiffing, LandscapeInput, LandscapeModeling,
    REPORT_REPRODUCIBILITY_STANDARD_ID, RawAmenableArtifact, RegistryDumpIngestion,
    run_belvedere_landscape_diff,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn landscape_diff_surfaces_snapshot_changes_for_auditors() -> Result<()> {
    belvedere::init_tracing();

    let previous = fixture_landscape(Vec::new())?;
    let current = fixture_landscape(include_bytes!("fixtures/proof_assessments.jsonl").to_vec())?;
    let (diff, receipt) = LandscapeDiffing
        .diff(LandscapeDiffInput::new(previous, current))
        .into_diagnostic()
        .wrap_err("landscape diff should produce deterministic auditor evidence")?;

    assert_eq!(receipt.counts().added_items(), 4);
    assert_eq!(receipt.counts().removed_items(), 1);
    assert_eq!(receipt.counts().changed_items(), 1);
    assert_eq!(receipt.counts().total_items(), diff.items().len());
    assert_landscape_diff_receipt_identity(&receipt);
    assert_eq!(
        receipt.subject_kind_counts().get("diff_subject_claim"),
        Some(&2)
    );
    assert_eq!(
        receipt.subject_kind_counts().get("diff_subject_assessment"),
        Some(&2)
    );
    assert_eq!(
        receipt.subject_kind_counts().get("diff_subject_gap"),
        Some(&2)
    );
    assert_eq!(
        receipt.item_kind_counts().get("diff_item_kind_added"),
        Some(&4)
    );
    assert_eq!(
        receipt.item_kind_counts().get("diff_item_kind_changed"),
        Some(&1)
    );
    assert_eq!(
        receipt.item_kind_counts().get("diff_item_kind_removed"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .change_subject_counts()
            .get("diff_change_added_assessment"),
        Some(&2)
    );
    assert_eq!(
        receipt.change_subject_counts().get("diff_change_added_gap"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .change_subject_counts()
            .get("diff_change_changed_claim"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .change_subject_counts()
            .get("diff_change_removed_gap"),
        Some(&1)
    );
    assert_eq!(
        receipt.trace_source_kind_counts(),
        &trace_source_kind_counts(diff.items())
    );
    assert_eq!(
        receipt.trace_source_counts(),
        &trace_source_counts(diff.items())
    );
    assert_eq!(
        receipt.trace_source_record_ids(),
        &trace_source_record_ids(diff.items())
    );
    assert_eq!(receipt.diff_item_ids().len(), diff.items().len());
    assert!(
        diff.items()
            .iter()
            .all(|item| receipt.diff_item_ids().contains(item.item_id()))
    );
    assert!(
        receipt
            .trace_source_kind_counts()
            .get("diff_trace_source_kind_assessment")
            .is_some_and(|count| *count > 0)
    );
    assert!(
        receipt
            .trace_source_kind_counts()
            .get("diff_trace_source_kind_kani_proof")
            .is_some_and(|count| *count > 0)
    );

    let certificate = BelvedereStageCertificate::landscape_diff(&receipt);
    assert_eq!(certificate.stage(), BelvederePipelineStage::LandscapeDiff);
    assert_eq!(certificate.contract(), "belvedere.diff.landscape");
    assert!(certificate.input_digests().is_empty());
    assert!(
        certificate
            .standards()
            .contains(&LANDSCAPE_CONSTRUCTION_STANDARD_ID.to_owned())
    );
    assert!(
        certificate
            .standards()
            .contains(&REPORT_REPRODUCIBILITY_STANDARD_ID.to_owned())
    );
    assert_eq!(
        certificate.counts().get("total_items"),
        Some(&diff.items().len())
    );
    assert_eq!(certificate.counts().get("diff_item_kind_added"), Some(&4));
    assert_eq!(certificate.counts().get("diff_item_kind_changed"), Some(&1));
    assert_eq!(certificate.counts().get("diff_item_kind_removed"), Some(&1));
    assert!(
        receipt
            .diff_item_ids()
            .iter()
            .all(|diff_item_id| certificate.output_ids().contains(diff_item_id))
    );
    assert_eq!(certificate.counts().get("diff_subject_gap"), Some(&2));
    assert_eq!(
        certificate.counts().get("diff_change_added_assessment"),
        Some(&2)
    );
    assert_eq!(
        certificate.counts().get("diff_trace_source_links"),
        receipt.trace_source_counts().get("diff_trace_source_links")
    );
    assert_eq!(
        certificate
            .counts()
            .get("diff_trace_source_kind_assessment"),
        receipt
            .trace_source_kind_counts()
            .get("diff_trace_source_kind_assessment")
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .all(|check| check.passed())
    );
    assert!(
        certificate.invariant_checks().iter().any(|check| {
            check.name() == "landscape_diff_item_ids_classified" && check.passed()
        })
    );
    assert!(certificate.invariant_checks().iter().any(|check| {
        check.name() == "landscape_diff_core_counts_match_items" && check.passed()
    }));

    let assessment = diff
        .items()
        .iter()
        .find(|item| {
            item.item_id() == "belvedere.diff.Added.assessment:assess-belvedere-ingestion-1"
        })
        .ok_or_else(|| miette!("new assessment should be diff evidence"))?;
    assert_eq!(assessment.change_kind(), LandscapeDiffChangeKind::Added);
    assert_eq!(assessment.subject_kind(), "assessment");
    assert_eq!(assessment.subject_id(), "assess-belvedere-ingestion-1");
    assert!(assessment.summary().contains("recommendation Strengthen"));
    assert!(
        assessment
            .trace()
            .source_record_ids()
            .contains(&"assessment:assess-belvedere-ingestion-1".to_owned())
    );
    assert!(
        assessment
            .trace()
            .source_record_ids()
            .contains(&"kani_proof:belvedere::ingestion::record_identity_is_preserved".to_owned())
    );
    assert!(
        !assessment.trace().source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );

    let claim = diff
        .items()
        .iter()
        .find(|item| {
            item.item_id()
                == "belvedere.diff.Changed.claim:belvedere::ingestion::record_identity_is_preserved"
        })
        .ok_or_else(|| miette!("assessment attachment should change proof claim evidence"))?;
    assert_eq!(claim.change_kind(), LandscapeDiffChangeKind::Changed);
    assert!(
        claim
            .summary()
            .contains("to 0 bases, 1 verifier proofs, 1 assessments")
    );
    assert!(
        claim
            .trace()
            .source_record_ids()
            .contains(&"assessment:assess-belvedere-ingestion-1".to_owned())
    );
    assert!(
        claim
            .trace()
            .source_record_ids()
            .contains(&"kani_proof:belvedere::ingestion::record_identity_is_preserved".to_owned())
    );
    assert!(
        !claim
            .trace()
            .source_record_ids()
            .contains(&"evidence_link:belvedere::ingestion::DecodedRegistrySnapshot#0".to_owned())
    );

    let removed_gap = diff
        .items()
        .iter()
        .find(|item| {
            item.item_id()
                == "belvedere.diff.Removed.gap:belvedere::ingestion::record_identity_is_preserved:MissingAssessment"
        })
        .ok_or_else(|| miette!("resolved missing-assessment gap should be removed"))?;
    assert_eq!(removed_gap.change_kind(), LandscapeDiffChangeKind::Removed);
    assert_eq!(
        removed_gap.trace().claim_id(),
        "belvedere::ingestion::record_identity_is_preserved"
    );
    assert_eq!(
        removed_gap.trace().source_record_ids(),
        &["kani_proof:belvedere::ingestion::record_identity_is_preserved".to_owned()]
    );

    let unsupported_claim = diff
        .items()
        .iter()
        .find(|item| {
            item.item_id()
                == "belvedere.diff.Added.claim:belvedere::ingestion::unsupported_fields_are_accounted_for"
        })
        .ok_or_else(|| miette!("assessment-only proof claim should be added"))?;
    assert!(unsupported_claim.summary().contains("added claim"));
    assert_eq!(
        unsupported_claim.trace().source_record_ids(),
        &[
            "assessment:belvedere::ingestion::unsupported_fields_are_accounted_for@1790550060"
                .to_owned()
        ]
    );
    assert!(diff.items().iter().any(|item| {
        item.item_id()
            == "belvedere.diff.Added.gap:belvedere::ingestion::unsupported_fields_are_accounted_for:MissingProof"
            && item.summary().contains("added gap MissingProof")
    }));
    Ok(())
}

fn assert_landscape_diff_receipt_identity(receipt: &belvedere::LandscapeDiffReceipt) {
    assert!(
        receipt
            .receipt_id()
            .starts_with("belvedere.receipt.landscape_diff:")
    );
    assert_eq!(
        receipt.receipt_id(),
        &format!(
            "belvedere.receipt.landscape_diff:{}",
            receipt.receipt_digest()
        )
    );
}

#[test]
fn landscape_diff_runner_uses_the_normal_artifact_path() -> Result<()> {
    belvedere::init_tracing();

    let previous_assessments_path = std::env::temp_dir().join(format!(
        "belvedere-empty-assessments-{}.jsonl",
        process::id()
    ));
    fs::write(&previous_assessments_path, [])
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be writable")?;

    let previous_input = BelvedereRunInput::new(
        "tests/fixtures/registry_dump.json",
        previous_assessments_path.clone(),
    );
    let current_input = BelvedereRunInput::new(
        "tests/fixtures/registry_dump.json",
        "tests/fixtures/proof_assessments.jsonl",
    );
    let report = run_belvedere_landscape_diff(BelvedereLandscapeDiffRunInput::new(
        previous_input,
        current_input,
    ))
    .into_diagnostic()
    .wrap_err("normal artifact paths should produce a certified landscape diff")?;

    assert_eq!(
        report
            .previous_assessment_receipt()
            .source_record_ids()
            .len(),
        0
    );
    assert_eq!(
        report
            .current_assessment_receipt()
            .source_record_ids()
            .len(),
        2
    );
    assert_eq!(report.previous_landscape_receipt().claim_count(), 3);
    assert_eq!(report.current_landscape_receipt().claim_count(), 4);
    assert_eq!(report.diff_receipt().counts().total_items(), 6);
    assert_eq!(
        report.previous_view_model_receipt().counts().claim_cards(),
        3
    );
    assert_eq!(
        report
            .previous_view_model_receipt()
            .counts()
            .review_workflow_items(),
        3
    );
    assert_eq!(
        report
            .previous_view_model_receipt()
            .counts()
            .claim_detail_items(),
        3
    );
    assert_eq!(report.previous_ui_ir_receipt().counts().section_count(), 13);
    assert!(
        report.previous_projection().trace_widgets().len()
            >= report.previous_view_model_receipt().counts().claim_cards()
    );
    assert_eq!(
        report.current_view_model_receipt().counts().claim_cards(),
        4
    );
    assert_eq!(
        report
            .current_view_model_receipt()
            .counts()
            .review_workflow_items(),
        4
    );
    assert_eq!(
        report
            .current_view_model_receipt()
            .counts()
            .claim_detail_items(),
        4
    );
    assert_eq!(report.current_ui_ir_receipt().counts().section_count(), 13);
    assert!(
        report.current_projection().trace_widgets().len()
            >= report.current_view_model_receipt().counts().claim_cards()
    );
    assert_eq!(
        report
            .diff_receipt()
            .subject_kind_counts()
            .get("diff_subject_assessment"),
        Some(&2)
    );
    assert_eq!(
        report.diff_certificate().stage(),
        BelvederePipelineStage::LandscapeDiff
    );
    assert_eq!(
        report.diff_certificate().counts().get("total_items"),
        Some(&report.diff().items().len())
    );
    assert!(report.diff().items().iter().all(|item| {
        report
            .diff_receipt()
            .diff_item_ids()
            .contains(item.item_id())
    }));
    assert!(
        report
            .diff_receipt()
            .diff_item_ids()
            .iter()
            .all(|diff_item_id| report
                .diff_certificate()
                .output_ids()
                .contains(diff_item_id))
    );
    assert_eq!(
        report
            .diff_certificate()
            .counts()
            .get("diff_subject_assessment"),
        Some(&2)
    );
    assert_eq!(
        report
            .diff_certificate()
            .counts()
            .get("diff_change_added_assessment"),
        Some(&2)
    );
    assert_eq!(
        report
            .diff_certificate()
            .counts()
            .get("diff_trace_source_kind_assessment"),
        report
            .diff_receipt()
            .trace_source_kind_counts()
            .get("diff_trace_source_kind_assessment")
    );
    assert_eq!(
        report
            .diff_certificate()
            .counts()
            .get("diff_trace_source_links"),
        report
            .diff_receipt()
            .trace_source_counts()
            .get("diff_trace_source_links")
    );
    assert!(
        report
            .diff_certificate()
            .invariant_checks()
            .iter()
            .any(
                |check| check.name() == "landscape_diff_trace_sources_admissible" && check.passed()
            )
    );
    assert_eq!(
        report.certificate().schema_id(),
        BELVEDERE_RUN_CERTIFICATE_SCHEMA_ID
    );
    assert_eq!(report.certificate().input_artifact_ids().len(), 4);
    assert_eq!(report.certificate().stage_certificates().len(), 11);
    assert_eq!(report.certificate().stage_certificate_ids().len(), 11);
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_registry_ingestion"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_assessment_ingestion"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_landscape_modeling"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_auditor_navigation"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_ui_ir_projection"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .stage_kind_counts()
            .get("stage_landscape_diff"),
        Some(&1)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_ingestion"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_landscape"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_view_model"),
        Some(&1)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_ui_ir"),
        Some(&2)
    );
    assert_eq!(
        report
            .certificate()
            .pipeline_claim_phase_counts()
            .get("phase_run"),
        Some(&1)
    );
    assert!(
        report
            .manifest()
            .invariant_checks()
            .iter()
            .all(|check| check.passed())
    );
    assert_eq!(
        report
            .manifest()
            .stage_kind_counts()
            .get("manifest_stage_kind_registry_ingestion"),
        Some(&2)
    );
    assert_eq!(
        report
            .manifest()
            .stage_kind_counts()
            .get("manifest_stage_kind_landscape_diff"),
        Some(&1)
    );
    assert_eq!(
        report
            .manifest()
            .output_kind_counts()
            .get("manifest_output_kind_landscape_diff_receipt"),
        Some(&1)
    );
    assert!(
        report
            .certificate()
            .input_artifact_ids()
            .iter()
            .any(|artifact_id| artifact_id.starts_with("previous_registry:registry_dump:"))
    );
    assert!(
        report
            .certificate()
            .input_artifact_ids()
            .iter()
            .any(|artifact_id| artifact_id.starts_with("current_assessment:proof_assessments:"))
    );
    assert!(
        report
            .certificate()
            .stage_certificate_ids()
            .contains(report.diff_certificate().id())
    );
    assert!(report.certificate().stage_certificates().iter().any(
        |certificate| certificate.stage() == BelvederePipelineStage::AuditorNavigation
            && certificate.counts().get("review_workflow_task_total") == Some(&3)
    ));
    assert!(report.certificate().stage_certificates().iter().any(
        |certificate| certificate.stage() == BelvederePipelineStage::AuditorNavigation
            && certificate.counts().get("review_workflow_task_total") == Some(&4)
    ));
    assert!(report.certificate().stage_certificates().iter().any(
        |certificate| certificate.stage() == BelvederePipelineStage::UiIrProjection
            && certificate.counts().get("ui_ir_sections_populated") == Some(&13)
    ));

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
}

fn trace_source_kind_counts(items: &[belvedere::LandscapeDiffItem]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for source_record_id in items
        .iter()
        .flat_map(|item| item.trace().source_record_ids())
    {
        let kind = source_record_id
            .split_once(':')
            .map_or(source_record_id.as_str(), |(kind, _identity)| kind);
        *counts
            .entry(format!("diff_trace_source_kind_{kind}"))
            .or_default() += 1;
    }
    counts
}

fn trace_source_counts(items: &[belvedere::LandscapeDiffItem]) -> BTreeMap<String, usize> {
    let trace_source_record_ids = trace_source_record_ids(items);
    let items_with_sources = items
        .iter()
        .filter(|item| !item.trace().source_record_ids().is_empty())
        .count();
    BTreeMap::from([
        ("diff_traceable_items".to_owned(), items.len()),
        (
            "diff_trace_items_with_sources".to_owned(),
            items_with_sources,
        ),
        (
            "diff_trace_source_links".to_owned(),
            items
                .iter()
                .map(|item| item.trace().source_record_ids().len())
                .sum(),
        ),
        (
            "diff_trace_unique_source_records".to_owned(),
            trace_source_record_ids.len(),
        ),
    ])
}

fn trace_source_record_ids(items: &[belvedere::LandscapeDiffItem]) -> Vec<String> {
    let mut source_record_ids = items
        .iter()
        .flat_map(|item| item.trace().source_record_ids().iter().cloned())
        .collect::<Vec<_>>();
    source_record_ids.sort();
    source_record_ids.dedup();
    source_record_ids
}

fn fixture_landscape(assessment_bytes: Vec<u8>) -> Result<belvedere::LandscapeGraph> {
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

    let (registry, registry_receipt) = RegistryDumpIngestion
        .ingest(registry_artifact)
        .into_diagnostic()
        .wrap_err("registry fixture should satisfy ingestion contract")?;
    let (assessments, assessment_receipt) = AssessmentStreamIngestion
        .ingest(assessment_artifact)
        .into_diagnostic()
        .wrap_err("assessment fixture should satisfy ingestion contract")?;
    let landscape_input = LandscapeInput::from_ingestion(
        registry,
        &registry_receipt,
        assessments,
        &assessment_receipt,
    );
    let (landscape, _landscape_receipt) = LandscapeModeling
        .model(landscape_input)
        .into_diagnostic()
        .wrap_err("landscape modeling is conservative and infallible")?;

    Ok(landscape)
}
