use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, AuditorNavigationProjection, GapSeverity,
    LandscapeGapKind, LandscapeInput, LandscapeModeling, RawAmenableArtifact,
    RegistryDumpIngestion, ViewModelInput,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn view_model_projection_builds_auditor_navigation_evidence() -> Result<()> {
    belvedere::init_tracing();

    let (model, receipt) = AuditorNavigationProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("view-model projection is trace-preserving and infallible")?;

    assert_eq!(model.overview().claim_count(), 4);
    assert_eq!(model.overview().root_count(), 3);
    assert_eq!(model.overview().proof_coverage_count(), 2);
    assert_eq!(model.overview().assessment_count(), 2);
    assert_eq!(model.overview().high_priority_gap_count(), 3);

    assert_eq!(receipt.counts().claim_cards(), 4);
    assert_eq!(receipt.counts().filter_facets(), 10);
    assert_eq!(receipt.counts().backend_coverage_items(), 12);
    assert_eq!(receipt.counts().proof_chain_items(), 4);
    assert_eq!(receipt.counts().assessment_items(), 2);
    assert_eq!(receipt.counts().assessment_work_items(), 2);
    assert_eq!(receipt.counts().witness_items(), 1);
    assert_eq!(receipt.counts().root_items(), 3);
    assert_eq!(receipt.counts().source_items(), 7);

    let decoded_item = model
        .claim_explorer()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot")
        .ok_or_else(|| miette!("decoded registry claim should be visible in claim explorer"))?;
    assert!(decoded_item.verifiers().contains(&"kani".to_owned()));
    assert_eq!(decoded_item.contract_count(), 1);
    assert_eq!(decoded_item.gap_count(), 2);
    assert!(
        decoded_item.trace().source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );
    assert!(
        decoded_item
            .filter_ids()
            .contains(&"belvedere.filter.claim_family:belvedere::ingestion".to_owned())
    );
    assert!(
        decoded_item
            .filter_ids()
            .contains(&"belvedere.filter.verifier:kani".to_owned())
    );
    assert!(
        decoded_item
            .filter_ids()
            .contains(&"belvedere.filter.support_kind:checked_with_trusted".to_owned())
    );
    assert!(
        decoded_item
            .filter_ids()
            .contains(&"belvedere.filter.gap_kind:MissingAssessment".to_owned())
    );
    assert!(
        decoded_item
            .filter_ids()
            .contains(&"belvedere.filter.gap_kind:TrustedOrOpaqueWitnessSupport".to_owned())
    );
    assert!(
        decoded_item
            .filter_ids()
            .contains(&"belvedere.filter.priority:high".to_owned())
    );

    let proof_item = model
        .claim_explorer()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| miette!("Kani proof claim should be visible in claim explorer"))?;
    assert!(
        proof_item
            .filter_ids()
            .contains(&"belvedere.filter.package:belvedere".to_owned())
    );
    assert!(
        proof_item
            .filter_ids()
            .contains(&"belvedere.filter.recommendation:Strengthen".to_owned())
    );

    let decoded_chain = model
        .proof_chains()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot")
        .ok_or_else(|| miette!("decoded registry claim should have a proof-chain item"))?;
    assert!(
        decoded_chain
            .root_ids()
            .contains(&"belvedere::ingestion::RawAmenableArtifact".to_owned())
    );
    assert_eq!(decoded_chain.review_state(), "elevated");
    assert_eq!(
        decoded_chain.next_action(),
        "review trusted or opaque witness support"
    );
    assert_eq!(decoded_chain.topology().transitive_claim_count(), 2);
    assert_eq!(decoded_chain.topology().max_depth(), 1);
    assert_eq!(decoded_chain.topology().branching_claim_count(), 0);
    assert_eq!(decoded_chain.topology().chain_gap_count(), 3);
    assert_eq!(decoded_chain.gap_count(), decoded_item.gap_count());
    Ok(())
}

#[test]
fn view_model_projection_builds_backend_coverage_items() -> Result<()> {
    belvedere::init_tracing();

    let (model, receipt) = AuditorNavigationProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("view-model projection should expose backend coverage")?;

    assert_eq!(
        receipt.counts().backend_coverage_items(),
        model.backend_coverage().len()
    );
    let covered = model
        .backend_coverage()
        .iter()
        .find(|item| {
            item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && item.verifier() == "kani"
        })
        .ok_or_else(|| miette!("Kani coverage should be visible for decoded registry"))?;
    assert!(covered.covered());

    let missing = model
        .backend_coverage()
        .iter()
        .find(|item| {
            item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && item.verifier() == "creusot"
        })
        .ok_or_else(|| miette!("missing Creusot coverage should be visible"))?;
    assert!(!missing.covered());
    assert!(
        missing.trace().source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );
    Ok(())
}

#[test]
fn view_model_projection_builds_source_navigation_items() -> Result<()> {
    belvedere::init_tracing();

    let (model, receipt) = AuditorNavigationProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("view-model projection should expose source navigation")?;

    assert_eq!(
        receipt.counts().source_items(),
        model.source_navigation().len()
    );
    let proof_record = model
        .source_navigation()
        .iter()
        .find(|item| {
            item.source_record_id()
                == "proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani"
        })
        .ok_or_else(|| miette!("proof source record should be navigable"))?;
    assert_eq!(
        proof_record.label(),
        "Proof record belvedere::ingestion::DecodedRegistrySnapshot@kani"
    );
    assert_eq!(proof_record.source_kind(), "proof_record");
    assert_eq!(
        proof_record.target(),
        "belvedere::ingestion::DecodedRegistrySnapshot@kani"
    );
    assert_eq!(proof_record.trace().source_record_ids().len(), 1);
    assert!(
        proof_record.trace().source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );

    let kani_proof = model
        .source_navigation()
        .iter()
        .find(|item| {
            item.source_record_id()
                == "kani_proof:belvedere::ingestion::record_identity_is_preserved"
        })
        .ok_or_else(|| miette!("Kani proof source record should be navigable"))?;
    assert_eq!(kani_proof.source_kind(), "kani_proof");
    assert_eq!(
        kani_proof.target(),
        "belvedere::ingestion::record_identity_is_preserved"
    );
    Ok(())
}

#[test]
fn view_model_projection_builds_auditor_filter_facets() -> Result<()> {
    belvedere::init_tracing();

    let (model, receipt) = AuditorNavigationProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("view-model projection should expose auditor filter facets")?;

    assert_eq!(
        receipt.counts().filter_facets(),
        model.filter_facets().len()
    );

    let verifier = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.verifier:kani")
        .ok_or_else(|| miette!("kani verifier facet should be available"))?;
    assert_eq!(verifier.group(), "verifier");
    assert_eq!(verifier.value(), "kani");
    assert_eq!(verifier.count(), 2);
    assert_eq!(verifier.trace().claim_id(), verifier.facet_id());

    let claim_family = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.claim_family:belvedere::ingestion")
        .ok_or_else(|| miette!("claim-family facet should be available"))?;
    assert_eq!(claim_family.group(), "claim_family");
    assert_eq!(claim_family.label(), "Claim family belvedere::ingestion");
    assert_eq!(claim_family.count(), 4);

    let strengthen = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.recommendation:Strengthen")
        .ok_or_else(|| miette!("strengthen recommendation facet should be available"))?;
    assert_eq!(strengthen.label(), "Recommendation Strengthen");
    assert_eq!(strengthen.count(), 1);

    let missing_proof = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.gap_kind:MissingProof")
        .ok_or_else(|| miette!("missing-proof gap facet should be available"))?;
    assert_eq!(missing_proof.count(), 2);

    let support_kind = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.support_kind:checked_with_trusted")
        .ok_or_else(|| miette!("support-kind facet should be available"))?;
    assert_eq!(support_kind.label(), "Support kind checked_with_trusted");
    assert_eq!(support_kind.count(), 1);

    let package = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.package:belvedere")
        .ok_or_else(|| miette!("package facet should be available"))?;
    assert_eq!(package.label(), "Package belvedere");
    assert_eq!(package.count(), 1);

    let priority = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.priority:high")
        .ok_or_else(|| miette!("high-priority facet should be available"))?;
    assert_eq!(priority.label(), "Priority high");
    assert_eq!(priority.count(), 3);
    Ok(())
}

#[test]
fn view_model_projection_preserves_review_queues_and_traceability() -> Result<()> {
    belvedere::init_tracing();

    let (model, _receipt) = AuditorNavigationProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("view-model projection is trace-preserving and infallible")?;

    let assessment = model
        .assessment_queue()
        .iter()
        .find(|item| item.assessment_id() == "assess-belvedere-ingestion-1")
        .ok_or_else(|| miette!("current assessment should be queued"))?;
    assert_eq!(
        assessment.proof_id(),
        "belvedere::ingestion::record_identity_is_preserved"
    );
    assert!(
        assessment
            .weak_dimensions()
            .contains(&"assertion_strength".to_owned())
    );
    assert!(
        assessment
            .trace()
            .source_record_ids()
            .contains(&"assessment:assess-belvedere-ingestion-1".to_owned())
    );

    let missing_assessment = model
        .assessment_work()
        .iter()
        .find(|item| {
            item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && item.action() == "assess proof coverage"
        })
        .ok_or_else(|| miette!("missing-assessment work should be queued"))?;
    assert_eq!(
        missing_assessment.reason(),
        "claim has proof coverage but no assessment"
    );
    assert_eq!(missing_assessment.queue_lane(), "assessment");
    assert_eq!(
        missing_assessment.resolution_label(),
        "collect proof assessment"
    );
    assert!(
        missing_assessment.trace().source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );

    let strengthen = model
        .assessment_work()
        .iter()
        .find(|item| {
            item.claim_id() == "belvedere::ingestion::record_identity_is_preserved"
                && item.action() == "strengthen proof"
        })
        .ok_or_else(|| miette!("strengthening work should be queued"))?;
    assert_eq!(strengthen.queue_lane(), "strengthening");
    assert_eq!(strengthen.resolution_label(), "strengthen current proof");
    assert!(strengthen.reason().contains("assess-belvedere-ingestion-1"));

    let witness = &model.witness_surface()[0];
    assert_eq!(
        witness.claim_id(),
        "belvedere::ingestion::DecodedRegistrySnapshot"
    );
    assert_eq!(witness.support_kind(), "checked_with_trusted");
    assert_eq!(witness.trivial(), 0);
    assert_eq!(witness.checked(), 1);
    assert_eq!(witness.trusted(), 1);

    let missing_assessment_gap = model
        .gaps()
        .iter()
        .find(|gap| {
            gap.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && gap.kind() == LandscapeGapKind::MissingAssessment
        })
        .ok_or_else(|| miette!("missing-assessment gap should be visible"))?;
    assert_eq!(missing_assessment_gap.severity(), GapSeverity::Review);
    assert_eq!(missing_assessment_gap.action(), "assess proof coverage");
    assert!(!missing_assessment_gap.high_priority());

    let missing_proof_gap = model
        .gaps()
        .iter()
        .find(|gap| gap.kind() == LandscapeGapKind::MissingProof)
        .ok_or_else(|| miette!("missing-proof gap should be visible"))?;
    assert_eq!(missing_proof_gap.severity(), GapSeverity::Blocking);
    assert_eq!(
        missing_proof_gap.action(),
        "establish modeled proof coverage"
    );
    assert!(missing_proof_gap.high_priority());

    let witness_gap = model
        .gaps()
        .iter()
        .find(|gap| gap.kind() == LandscapeGapKind::TrustedOrOpaqueWitnessSupport)
        .ok_or_else(|| miette!("trusted witness gap should be visible"))?;
    assert_eq!(witness_gap.severity(), GapSeverity::Elevated);
    assert_eq!(
        witness_gap.action(),
        "review trusted or opaque witness support"
    );
    assert!(witness_gap.high_priority());
    let raw_root = model
        .provenance_roots()
        .iter()
        .find(|root| root.claim_id() == "belvedere::ingestion::RawAmenableArtifact")
        .ok_or_else(|| miette!("raw artifact root should be cataloged"))?;
    assert_eq!(raw_root.root_kind(), "unmodeled_root");
    assert_eq!(raw_root.verifier_count(), 0);
    assert_eq!(raw_root.assessment_count(), 0);
    assert_eq!(raw_root.gap_count(), 1);

    let proof_root = model
        .provenance_roots()
        .iter()
        .find(|root| root.claim_id() == "belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| miette!("Kani proof root should be cataloged"))?;
    assert_eq!(proof_root.root_kind(), "proof_registration_root");
    assert_eq!(proof_root.verifier_count(), 1);
    assert_eq!(proof_root.assessment_count(), 1);
    assert_eq!(proof_root.gap_count(), 0);
    Ok(())
}

#[test]
fn view_model_projection_exposes_assessment_recency() -> Result<()> {
    belvedere::init_tracing();

    let (model, _receipt) = AuditorNavigationProjection
        .project(fixture_input_with_assessments(
            STALE_ASSESSMENTS.as_bytes().to_vec(),
        )?)
        .into_diagnostic()
        .wrap_err("view-model projection should expose assessment recency")?;

    let current = model
        .assessment_queue()
        .iter()
        .find(|item| item.assessment_id() == "assess-current")
        .ok_or_else(|| miette!("current assessment should be queued"))?;
    assert!(current.current());
    assert_eq!(current.superseded_count(), 1);

    let older = model
        .assessment_queue()
        .iter()
        .find(|item| item.assessment_id() == "assess-old")
        .ok_or_else(|| miette!("older assessment should be queued"))?;
    assert!(!older.current());
    assert_eq!(older.superseded_count(), 0);
    Ok(())
}

fn fixture_input() -> Result<ViewModelInput> {
    fixture_input_with_assessments(include_bytes!("fixtures/proof_assessments.jsonl").to_vec())
}

fn fixture_input_with_assessments(assessment_bytes: Vec<u8>) -> Result<ViewModelInput> {
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

    Ok(ViewModelInput::new(landscape))
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
