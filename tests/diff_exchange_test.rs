use std::{fs, process};

use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, BelvedereLandscapeDiffRunInput,
    BelvederePipelineStage, BelvedereRunInput, BelvedereStageCertificate, LandscapeDiffChangeKind,
    LandscapeDiffInput, LandscapeDiffing, LandscapeInput, LandscapeModeling, RawAmenableArtifact,
    RegistryDumpIngestion, run_belvedere_landscape_diff,
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

    let certificate = BelvedereStageCertificate::landscape_diff(&receipt);
    assert_eq!(certificate.stage(), BelvederePipelineStage::LandscapeDiff);
    assert_eq!(certificate.contract(), "belvedere.diff.landscape");
    assert_eq!(
        certificate.counts().get("total_items"),
        Some(&diff.items().len())
    );
    assert_eq!(certificate.counts().get("diff_subject_gap"), Some(&2));
    assert_eq!(
        certificate.counts().get("diff_change_added_assessment"),
        Some(&2)
    );
    assert!(
        certificate
            .invariant_checks()
            .iter()
            .all(|check| check.passed())
    );

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

    assert!(diff.items().iter().any(|item| {
        item.item_id()
            == "belvedere.diff.Added.claim:belvedere::ingestion::unsupported_fields_are_accounted_for"
            && item.summary().contains("added claim")
    }));
    assert!(diff.items().iter().any(|item| {
        item.item_id()
            == "belvedere.diff.Added.gap:belvedere::ingestion::unsupported_fields_are_accounted_for:MissingProof"
            && item.summary().contains("added gap MissingProof")
    }));
    Ok(())
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

    fs::remove_file(previous_assessments_path)
        .into_diagnostic()
        .wrap_err("temporary empty assessment artifact should be removable")?;
    Ok(())
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

    Ok(landscape)
}
