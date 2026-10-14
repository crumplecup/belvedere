use std::collections::BTreeSet;

use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, AuditorNavigationProjection, GapSeverity,
    LandscapeGapKind, LandscapeInput, LandscapeModeling, RawAmenableArtifact,
    RegistryDumpIngestion, ReviewWorkflowPriority, ViewModelInput,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn contract_fragments_keep_distinct_source_navigation() -> Result<()> {
    belvedere::init_tracing();

    let mut registry: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    let contracts = registry["contract_records"]
        .as_array_mut()
        .ok_or_else(|| miette!("contract fixture should be an array"))?;
    let mut second = contracts[0].clone();
    second["fragment"] = "second_obligation".into();
    contracts.push(second);
    let input = fixture_input_with_artifacts(
        serde_json::to_vec(&registry).into_diagnostic()?,
        include_bytes!("fixtures/proof_assessments.jsonl").to_vec(),
    )?;
    let (model, _) = AuditorNavigationProjection
        .project(input)
        .into_diagnostic()?;
    let sources = model
        .source_navigation()
        .iter()
        .filter(|source| source.source_kind() == "contract_record")
        .collect::<Vec<_>>();
    assert_eq!(sources.len(), 2);
    assert_ne!(sources[0].source_record_id(), sources[1].source_record_id());
    assert_ne!(sources[0].source_locator(), sources[1].source_locator());
    assert!(
        sources
            .iter()
            .all(|source| source.source_locator().contains("fragment_sha256="))
    );
    Ok(())
}

#[test]
fn kani_registrations_keep_distinct_source_navigation_and_gap_ids() -> Result<()> {
    belvedere::init_tracing();

    let mut registry: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    let proofs = registry["kani_proofs"]
        .as_array_mut()
        .ok_or_else(|| miette!("Kani proof fixture should be an array"))?;
    for harness in ["verify_secondary", "verify_tertiary"] {
        let mut proof = proofs[0].clone();
        proof["harness"] = harness.into();
        proof["status"] = "failed".into();
        proofs.push(proof);
    }
    let input = fixture_input_with_artifacts(
        serde_json::to_vec(&registry).into_diagnostic()?,
        include_bytes!("fixtures/proof_assessments.jsonl").to_vec(),
    )?;
    let (model, _) = AuditorNavigationProjection
        .project(input)
        .into_diagnostic()?;
    let source_records = model
        .source_navigation()
        .iter()
        .filter(|source| source.source_kind() == "kani_proof")
        .collect::<Vec<_>>();
    assert_eq!(source_records.len(), 3);
    assert!(
        source_records
            .iter()
            .any(|source| source.source_locator().contains("harness=verify_secondary"))
    );
    assert!(
        source_records
            .iter()
            .any(|source| source.source_locator().contains("harness=verify_tertiary"))
    );
    let gap_ids = model
        .gaps()
        .iter()
        .filter(|gap| gap.kind() == LandscapeGapKind::ProofExecutionUnresolved)
        .map(|gap| gap.gap_id().as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(gap_ids.len(), 2);
    assert!(
        gap_ids
            .iter()
            .any(|id| id.ends_with("belvedere@verify_secondary"))
    );
    assert!(
        gap_ids
            .iter()
            .any(|id| id.ends_with("belvedere@verify_tertiary"))
    );
    Ok(())
}

#[test]
fn witness_gaps_keep_distinct_origin_ids_through_navigation() -> Result<()> {
    belvedere::init_tracing();

    let mut registry: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/registry_dump.json")).into_diagnostic()?;
    let witnesses = registry["witness_export_records"]
        .as_array_mut()
        .ok_or_else(|| miette!("witness fixture should be an array"))?;
    let mut second = witnesses[0].clone();
    second["destination_module"] = "belvedere::proofs::secondary".into();
    witnesses.push(second);
    let input = fixture_input_with_artifacts(
        serde_json::to_vec(&registry).into_diagnostic()?,
        include_bytes!("fixtures/proof_assessments.jsonl").to_vec(),
    )?;
    let landscape_gaps = input
        .landscape()
        .gaps()
        .iter()
        .filter(|gap| gap.kind() == LandscapeGapKind::TrustedOrOpaqueWitnessSupport)
        .collect::<Vec<_>>();
    assert_eq!(landscape_gaps.len(), 2);
    assert_ne!(
        landscape_gaps[0].origin_key(),
        landscape_gaps[1].origin_key()
    );

    let (model, _) = AuditorNavigationProjection
        .project(input)
        .into_diagnostic()?;
    let gap_ids = model
        .gaps()
        .iter()
        .filter(|gap| gap.kind() == LandscapeGapKind::TrustedOrOpaqueWitnessSupport)
        .map(|gap| gap.gap_id().as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(gap_ids.len(), 2);
    assert!(
        gap_ids
            .iter()
            .any(|id| id.ends_with("kani@belvedere::proofs::ingestion"))
    );
    assert!(
        gap_ids
            .iter()
            .any(|id| id.ends_with("kani@belvedere::proofs::secondary"))
    );
    Ok(())
}

#[test]
fn view_model_projection_builds_auditor_navigation_evidence() -> Result<()> {
    belvedere::init_tracing();

    let input = fixture_input()?;
    let expected_source_landscape_receipt_digest =
        input.source_landscape_receipt_digest().to_owned();
    let (model, receipt) = AuditorNavigationProjection
        .project(input)
        .into_diagnostic()
        .wrap_err("view-model projection is trace-preserving and infallible")?;

    assert_eq!(model.overview().claim_count(), 4);
    assert_eq!(model.overview().root_count(), 3);
    assert_eq!(model.overview().proof_coverage_count(), 2);
    assert_eq!(model.overview().assessment_count(), 2);
    assert_eq!(model.overview().current_assessment_count(), 2);
    assert_eq!(model.overview().superseded_assessment_count(), 0);
    assert_eq!(model.overview().high_priority_gap_count(), 3);

    assert_eq!(receipt.counts().claim_cards(), 4);
    assert_eq!(receipt.counts().claim_detail_items(), 4);
    assert_eq!(receipt.counts().filter_facets(), 12);
    assert_eq!(receipt.counts().backend_coverage_items(), 12);
    assert_eq!(receipt.counts().proof_chain_items(), 4);
    assert_eq!(receipt.counts().review_workflow_items(), 4);
    assert_eq!(receipt.counts().assessment_history_items(), 2);
    assert_eq!(receipt.counts().assessment_items(), 2);
    assert_eq!(receipt.counts().assessment_work_items(), 2);
    assert_eq!(receipt.counts().witness_items(), 1);
    assert_eq!(receipt.counts().root_items(), 3);
    assert_eq!(receipt.counts().source_items(), 7);
    assert_view_model_receipt_identity(&receipt);
    assert!(
        receipt
            .source_landscape_receipt_id()
            .starts_with("belvedere.receipt.landscape:")
    );
    assert_eq!(
        receipt.source_landscape_receipt_digest(),
        &expected_source_landscape_receipt_digest
    );
    assert!(is_hex_digest(receipt.source_landscape_receipt_digest()));
    assert_eq!(
        receipt.lane_counts().get("assessment_queue_lane_accepted"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .lane_counts()
            .get("assessment_queue_lane_strengthening"),
        Some(&1)
    );
    assert_eq!(
        receipt.lane_counts().get("assessment_work_lane_assessment"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .lane_counts()
            .get("assessment_work_lane_strengthening"),
        Some(&1)
    );
    assert_eq!(receipt.lane_counts().get("gap_lane_assessment"), Some(&1));
    assert_eq!(
        receipt.lane_counts().get("gap_lane_proof_coverage"),
        Some(&2)
    );
    assert_eq!(
        receipt.lane_counts().get("gap_lane_witness_review"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .source_artifact_counts()
            .get("source_artifact_registry_dump"),
        Some(&5)
    );
    assert_eq!(
        receipt
            .source_artifact_counts()
            .get("source_artifact_proof_assessments"),
        Some(&2)
    );
    assert_eq!(
        receipt.source_kind_counts().get("source_kind_assessment"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .source_kind_counts()
            .get("source_kind_contract_record"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .source_kind_counts()
            .get("source_kind_evidence_link"),
        Some(&1)
    );
    assert_eq!(
        receipt.source_kind_counts().get("source_kind_kani_proof"),
        Some(&1)
    );
    assert_eq!(
        receipt.source_kind_counts().get("source_kind_proof_record"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .source_kind_counts()
            .get("source_kind_witness_export"),
        Some(&1)
    );
    assert_eq!(
        receipt.source_locator_counts().get("source_locator_items"),
        Some(&7)
    );
    assert_eq!(
        receipt
            .source_locator_counts()
            .get("source_locator_registry_dump"),
        Some(&5)
    );
    assert_eq!(
        receipt
            .source_locator_counts()
            .get("source_locator_proof_assessments"),
        Some(&2)
    );
    let total_transitive_claims: usize = model
        .proof_chains()
        .iter()
        .map(|item| item.topology().transitive_claim_count())
        .sum();
    let max_depth = model
        .proof_chains()
        .iter()
        .map(|item| item.topology().max_depth())
        .max()
        .unwrap_or_default();
    let chain_gaps: usize = model
        .proof_chains()
        .iter()
        .map(|item| item.topology().chain_gap_count())
        .sum();
    assert_eq!(
        receipt
            .proof_chain_topology_counts()
            .get("proof_chain_total_transitive_claims"),
        Some(&total_transitive_claims)
    );
    assert_eq!(
        receipt
            .proof_chain_topology_counts()
            .get("proof_chain_max_depth"),
        Some(&max_depth)
    );
    assert_eq!(
        receipt
            .proof_chain_topology_counts()
            .get("proof_chain_chain_gaps"),
        Some(&chain_gaps)
    );
    assert_eq!(
        receipt
            .provenance_root_kind_counts()
            .get("provenance_root_kind_assessment_only_root"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .provenance_root_kind_counts()
            .get("provenance_root_kind_proof_registration_root"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .provenance_root_kind_counts()
            .get("provenance_root_kind_unmodeled_root"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_proof_chain_state_blocked"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_proof_chain_state_elevated"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_proof_chain_state_ready"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_task_total"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_task_priority_blocking"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_task_priority_elevated"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_task_priority_ready"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_task_gap_links"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_tasks_with_gaps"),
        Some(&3)
    );
    assert_eq!(
        receipt.review_workflow_task_digests().len(),
        model.review_workflow().len()
    );
    assert!(receipt.review_workflow_task_digests().iter().all(|digest| {
        digest.len() == 64
            && digest
                .chars()
                .all(|character| character.is_ascii_hexdigit())
    }));
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_assessment_work_total"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_assessment_work_high_priority"),
        Some(&0)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_gap_severity_blocking"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .review_workflow_counts()
            .get("review_workflow_gap_high_priority"),
        Some(&3)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_filter_facet"),
        Some(&12)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_backend_coverage_item"),
        Some(&12)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_claim_card"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_claim_detail_item"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_proof_chain_item"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_review_workflow_item"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_assessment_history_item"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_assessment_queue_item"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_assessment_work_item"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_witness_item"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_provenance_root_item"),
        Some(&3)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_gap_item"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .trace_kind_counts()
            .get("view_model_trace_kind_source_navigation_item"),
        Some(&7)
    );
    assert_eq!(
        receipt
            .trace_source_counts()
            .get("view_model_traceable_items"),
        Some(&61)
    );
    assert_eq!(
        receipt
            .trace_source_counts()
            .get("view_model_trace_items_with_sources"),
        Some(&61)
    );
    assert_eq!(
        receipt
            .trace_source_counts()
            .get("view_model_trace_unique_source_records"),
        Some(&7)
    );
    assert_eq!(
        receipt
            .item_kind_counts()
            .get("view_model_item_kind_overview"),
        Some(&1)
    );
    assert_eq!(
        receipt.item_kind_counts().get("view_model_item_kind_claim"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .item_kind_counts()
            .get("view_model_item_kind_backend_coverage"),
        Some(&12)
    );
    assert_eq!(
        receipt.item_kind_counts().get("view_model_item_kind_gap"),
        Some(&4)
    );
    assert_eq!(
        receipt
            .item_kind_counts()
            .get("view_model_item_kind_source"),
        Some(&7)
    );
    assert_eq!(receipt.view_item_ids().len(), 62);
    assert!(
        receipt
            .view_item_ids()
            .contains(&"view_model.overview".to_owned())
    );
    assert!(
        receipt
            .view_item_ids()
            .contains(&"view_model.claim:belvedere::ingestion::DecodedRegistrySnapshot".to_owned())
    );
    assert!(receipt.view_item_ids().contains(
        &"view_model.claim_detail:belvedere::ingestion::DecodedRegistrySnapshot".to_owned()
    ));
    assert!(
        receipt.view_item_ids().contains(
            &"view_model.backend_coverage:belvedere::ingestion::DecodedRegistrySnapshot:kani"
                .to_owned()
        )
    );
    assert!(
        receipt
            .view_item_ids()
            .contains(&"view_model.source:assessment:assess-belvedere-ingestion-1".to_owned())
    );
    assert!(
        receipt.view_item_ids().contains(
            &"view_model.gap:belvedere::ingestion::DecodedRegistrySnapshot:missing_assessment"
                .to_owned()
        )
    );

    let decoded_item = model
        .claim_explorer()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot")
        .ok_or_else(|| miette!("decoded registry claim should be visible in claim explorer"))?;
    assert!(decoded_item.verifiers().contains(&"kani".to_owned()));
    assert_eq!(decoded_item.contract_count(), 1);
    assert_eq!(decoded_item.gap_count(), 2);
    assert_eq!(decoded_item.trace().source_record_ids().len(), 4);
    assert!(decoded_item.trace().source_record_ids().contains(
        &"contract_record:belvedere::ingestion::DecodedRegistrySnapshot@kani:ensures#8d167c786f51bd7c6bda8f92cb93b0baca7efb9204fea0d4d018fbf75c54126c".to_owned()
    ));
    assert!(
        decoded_item
            .trace()
            .source_record_ids()
            .contains(&"evidence_link:belvedere::ingestion::DecodedRegistrySnapshot#0".to_owned())
    );
    assert!(
        decoded_item.trace().source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );
    assert!(decoded_item.trace().source_record_ids().contains(
        &"witness_export:belvedere::ingestion::DecodedRegistrySnapshot@kani:belvedere::proofs::ingestion".to_owned()
    ));
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

    let decoded_detail = model
        .claim_details()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot")
        .ok_or_else(|| miette!("decoded registry claim should have a detail item"))?;
    assert!(!decoded_detail.root());
    assert!(
        decoded_detail
            .basis_ids()
            .contains(&"belvedere::ingestion::RawAmenableArtifact".to_owned())
    );
    assert!(
        decoded_detail
            .root_ids()
            .contains(&"belvedere::ingestion::RawAmenableArtifact".to_owned())
    );
    assert!(decoded_detail.verifiers().contains(&"kani".to_owned()));
    assert!(
        decoded_detail
            .contract_fragments()
            .iter()
            .any(|fragment| fragment.contains("kani ensures"))
    );
    assert!(decoded_detail.assessment_ids().is_empty());
    assert!(
        decoded_detail
            .gap_summaries()
            .iter()
            .any(|summary| summary.contains("MissingAssessment review: assess proof coverage"))
    );
    assert!(
        decoded_detail
            .gap_summaries()
            .iter()
            .any(|summary| summary.contains(
                "TrustedOrOpaqueWitnessSupport elevated: review trusted or opaque witness support"
            ))
    );
    assert_eq!(decoded_detail.review_state(), "elevated");
    assert_eq!(
        decoded_detail.next_action(),
        "review trusted or opaque witness support"
    );
    assert_eq!(decoded_detail.source_record_count(), 4);
    assert_eq!(decoded_detail.trace(), decoded_item.trace());

    let raw_root = model
        .provenance_roots()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::RawAmenableArtifact")
        .ok_or_else(|| miette!("raw artifact root should be visible in provenance catalog"))?;
    assert_eq!(raw_root.root_kind(), "unmodeled_root");
    assert!(raw_root.verifiers().is_empty());
    assert!(raw_root.assessment_ids().is_empty());
    assert!(raw_root.gap_summaries().iter().any(|summary| {
        summary.contains("MissingProof blocking: establish modeled proof coverage")
    }));
    assert_eq!(raw_root.next_action(), "establish modeled proof coverage");
    assert_eq!(
        raw_root.source_record_count(),
        raw_root.trace().source_record_ids().len()
    );
    assert_eq!(
        raw_root.source_record_ids(),
        raw_root.trace().source_record_ids()
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
    let proof_assessment = model
        .assessment_queue()
        .iter()
        .find(|item| item.proof_id() == proof_item.claim_id())
        .ok_or_else(|| miette!("proof claim should have an assessment queue entry"))?;
    assert!(proof_item.filter_ids().contains(&format!(
        "belvedere.filter.assessment_queue_group:{}",
        proof_assessment.queue_group_id()
    )));
    let proof_detail = model
        .claim_details()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| miette!("Kani proof claim should have a detail item"))?;
    assert!(proof_detail.proof_statuses().iter().any(|status| {
        status.contains("status verified") && status.contains("scope bounded harness")
    }));
    let proof_chain = model
        .proof_chains()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| miette!("Kani proof claim should have a proof-chain item"))?;
    assert!(proof_chain.proof_statuses().iter().any(|status| {
        status.contains("status verified") && status.contains("scope bounded harness")
    }));

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
    let decoded_task = model
        .review_workflow()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot")
        .ok_or_else(|| miette!("decoded registry claim should have a workflow task"))?;
    assert!(
        decoded_task.gap_ids().contains(
            &"view_model.gap:belvedere::ingestion::DecodedRegistrySnapshot:missing_assessment"
                .to_owned()
        )
    );
    assert!(decoded_task.gap_ids().contains(
        &"view_model.gap:belvedere::ingestion::DecodedRegistrySnapshot:trusted_or_opaque_witness_support:kani@belvedere::proofs::ingestion"
            .to_owned()
    ));
    assert_eq!(decoded_task.gap_ids().len(), 2);
    assert_eq!(decoded_chain.topology().transitive_claim_count(), 2);
    assert_eq!(decoded_chain.topology().max_depth(), 1);
    assert_eq!(decoded_chain.topology().branching_claim_count(), 0);
    assert_eq!(decoded_chain.topology().chain_gap_count(), 3);
    let kani_chain_coverage = decoded_chain
        .backend_coverage()
        .iter()
        .find(|coverage| coverage.verifier() == "kani")
        .ok_or_else(|| miette!("proof chain should expose Kani chain coverage"))?;
    assert!(!kani_chain_coverage.complete());
    assert_eq!(kani_chain_coverage.covered_claim_count(), 1);
    assert_eq!(kani_chain_coverage.missing_claim_count(), 1);
    let creusot_chain_coverage = decoded_chain
        .backend_coverage()
        .iter()
        .find(|coverage| coverage.verifier() == "creusot")
        .ok_or_else(|| miette!("proof chain should expose Creusot chain coverage"))?;
    assert!(!creusot_chain_coverage.complete());
    assert_eq!(creusot_chain_coverage.covered_claim_count(), 0);
    assert_eq!(creusot_chain_coverage.missing_claim_count(), 2);
    assert_eq!(decoded_chain.gap_count(), decoded_item.gap_count());

    let decoded_task = model
        .review_workflow()
        .iter()
        .find(|item| item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot")
        .ok_or_else(|| miette!("decoded registry claim should have a review task"))?;
    assert_eq!(decoded_task.priority(), ReviewWorkflowPriority::Elevated);
    assert_eq!(decoded_task.lane(), "elevated:witness-review");
    assert_eq!(
        decoded_task.action(),
        "review trusted or opaque witness support"
    );
    assert!(decoded_task.rationale().contains("MissingAssessment"));
    assert_eq!(
        decoded_task.source_count(),
        decoded_task.trace().source_record_ids().len()
    );
    assert_eq!(decoded_task.task_digest().len(), 64);
    assert!(
        decoded_task
            .task_digest()
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );
    assert!(
        receipt
            .review_workflow_task_digests()
            .contains(decoded_task.task_digest())
    );
    assert!(receipt.view_item_ids().contains(&format!(
        "view_model.review_task:{}",
        decoded_task.task_id()
    )));
    assert!(
        receipt.view_item_ids().contains(
            &"view_model.assessment_history:belvedere::ingestion::record_identity_is_preserved"
                .to_owned()
        )
    );
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
    assert_eq!(proof_record.artifact_kind(), ArtifactKind::RegistryDump);
    assert_eq!(proof_record.source_kind(), "proof_record");
    assert_eq!(
        proof_record.target(),
        "belvedere::ingestion::DecodedRegistrySnapshot@kani"
    );
    assert_eq!(
        proof_record.source_locator(),
        "registry_dump.proof_records[evidence=belvedere::ingestion::DecodedRegistrySnapshot, verifier=kani]"
    );
    assert_eq!(proof_record.trace().source_record_ids().len(), 1);
    assert!(
        proof_record.trace().source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );

    let assessment_source = model
        .source_navigation()
        .iter()
        .find(|item| item.source_record_id() == "assessment:assess-belvedere-ingestion-1")
        .ok_or_else(|| miette!("assessment source record should be navigable"))?;
    assert_eq!(
        assessment_source.artifact_kind(),
        ArtifactKind::ProofAssessments
    );
    assert_eq!(assessment_source.source_kind(), "assessment");
    assert_eq!(
        assessment_source.source_locator(),
        "assessment_jsonl.assessment_id=assess-belvedere-ingestion-1"
    );

    let kani_proof = model
        .source_navigation()
        .iter()
        .find(|item| {
            item.source_record_id()
                == "kani_proof:belvedere::ingestion::record_identity_is_preserved@belvedere:verify_record_identity_is_preserved"
        })
        .ok_or_else(|| miette!("Kani proof source record should be navigable"))?;
    assert_eq!(kani_proof.source_kind(), "kani_proof");
    assert_eq!(
        kani_proof.target(),
        "belvedere::ingestion::record_identity_is_preserved"
    );
    assert_eq!(
        kani_proof.source_locator(),
        "registry_dump.kani_proofs[id=belvedere::ingestion::record_identity_is_preserved, package=belvedere, harness=verify_record_identity_is_preserved]"
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
    let facet_ids = model
        .filter_facets()
        .iter()
        .map(|facet| facet.facet_id().as_str())
        .collect::<BTreeSet<_>>();
    for claim in model.claim_explorer() {
        for filter_id in claim.filter_ids() {
            assert!(
                facet_ids.contains(filter_id.as_str()),
                "claim {} refers to missing facet {filter_id}",
                claim.claim_id()
            );
        }
    }
    assert_eq!(
        receipt.facet_group_counts().get("facet_group_claim_family"),
        Some(&1)
    );
    assert_eq!(
        receipt.facet_group_counts().get("facet_group_gap_kind"),
        Some(&3)
    );
    assert_eq!(
        receipt
            .facet_group_counts()
            .get("facet_group_recommendation"),
        Some(&2)
    );
    assert_eq!(
        receipt
            .facet_group_counts()
            .get("facet_group_assessment_queue_group"),
        Some(&2)
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
    let verifier_sources = model
        .claim_explorer()
        .iter()
        .filter(|item| item.verifiers().contains(&"kani".to_owned()))
        .flat_map(|item| item.trace().source_record_ids().iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    assert_eq!(verifier.trace().source_record_ids(), &verifier_sources);

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

    let strengthening_group = model
        .filter_facets()
        .iter()
        .find(|facet| {
            facet.facet_id()
                == "belvedere.filter.assessment_queue_group:assessment_queue.strengthening:assertion_strength+adversarial_coverage"
        })
        .ok_or_else(|| miette!("strengthening assessment queue group facet should be available"))?;
    assert_eq!(strengthening_group.group(), "assessment_queue_group");
    assert_eq!(
        strengthening_group.label(),
        "Assessment queue group strengthening / assertion_strength, adversarial_coverage"
    );
    assert_eq!(strengthening_group.count(), 1);
    let strengthening_assessment = model
        .assessment_queue()
        .iter()
        .find(|item| item.queue_group_id() == strengthening_group.value())
        .ok_or_else(|| miette!("strengthening group should select an assessment"))?;
    assert_eq!(
        strengthening_group.trace().source_record_ids(),
        strengthening_assessment.trace().source_record_ids()
    );
    assert_eq!(strengthening_group.trace().source_record_ids().len(), 2);

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
    let priority_sources = model
        .gaps()
        .iter()
        .filter(|item| item.high_priority())
        .flat_map(|item| item.trace().source_record_ids().iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    assert_eq!(priority.trace().source_record_ids(), &priority_sources);
    Ok(())
}

#[test]
fn replacement_gap_traces_the_assessment_that_caused_it() -> Result<()> {
    belvedere::init_tracing();

    let input = fixture_input_with_assessments(
        include_bytes!("fixtures/replacement_assessments.jsonl").to_vec(),
    )?;
    let landscape_gap = input
        .landscape()
        .gaps()
        .iter()
        .find(|gap| gap.kind() == LandscapeGapKind::ReplacementRecommended)
        .ok_or_else(|| miette!("replacement assessment should produce a landscape gap"))?;
    assert_eq!(
        landscape_gap.triggering_assessment_id(),
        Some("assess-replace")
    );

    let (model, _) = AuditorNavigationProjection
        .project(input)
        .into_diagnostic()
        .wrap_err("replacement gap should project into auditor navigation")?;
    let gap = model
        .gaps()
        .iter()
        .find(|gap| gap.kind() == LandscapeGapKind::ReplacementRecommended)
        .ok_or_else(|| miette!("replacement gap should be visible"))?;
    assert!(gap.high_priority());
    assert!(
        gap.trace()
            .source_record_ids()
            .contains(&"assessment:assess-replace".to_owned())
    );
    assert!(
        !gap.trace()
            .source_record_ids()
            .contains(&"assessment:assess-old".to_owned())
    );

    let facet = model
        .filter_facets()
        .iter()
        .find(|facet| facet.facet_id() == "belvedere.filter.gap_kind:ReplacementRecommended")
        .ok_or_else(|| miette!("replacement gap facet should be visible"))?;
    assert_eq!(
        facet.trace().source_record_ids(),
        gap.trace().source_record_ids()
    );
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
    assert_eq!(assessment.queue_lane(), "strengthening");
    assert_eq!(
        assessment.queue_group_id(),
        "assessment_queue.strengthening:assertion_strength+adversarial_coverage"
    );
    assert_eq!(
        assessment.queue_group_label(),
        "strengthening / assertion_strength, adversarial_coverage"
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
    assert_eq!(assessment.trace().source_record_ids().len(), 2);
    assert!(
        assessment
            .trace()
            .source_record_ids()
            .contains(&"kani_proof:belvedere::ingestion::record_identity_is_preserved@belvedere:verify_record_identity_is_preserved".to_owned())
    );

    let history = model
        .assessment_history()
        .iter()
        .find(|item| item.proof_id() == "belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| miette!("proof assessment history should be grouped by proof ID"))?;
    assert_eq!(
        history.current_assessment_id(),
        "assess-belvedere-ingestion-1"
    );
    assert_eq!(history.history_count(), 1);
    assert_eq!(history.superseded_count(), 0);
    assert_eq!(
        history.recommendation_flow(),
        &vec!["Strengthen".to_owned()]
    );
    assert!(
        history
            .weak_dimensions()
            .contains(&"assertion_strength".to_owned())
    );
    assert_eq!(
        history.current_resolution_label(),
        "strengthen current proof"
    );
    assert!(
        history
            .current_comment()
            .contains("preserves the source IDs")
    );
    assert_eq!(history.trace().source_record_ids().len(), 2);

    let missing_assessment = model
        .assessment_work()
        .iter()
        .find(|item| {
            item.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && item.action() == "assess proof coverage"
        })
        .ok_or_else(|| miette!("missing-assessment work should be queued"))?;
    assert_eq!(
        missing_assessment.work_id(),
        "assessment_work.missing_assessment:view_model.gap:belvedere::ingestion::DecodedRegistrySnapshot:missing_assessment"
    );
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
    assert_eq!(missing_assessment.trace().source_record_ids().len(), 4);

    let strengthen = model
        .assessment_work()
        .iter()
        .find(|item| {
            item.claim_id() == "belvedere::ingestion::record_identity_is_preserved"
                && item.action() == "strengthen proof"
        })
        .ok_or_else(|| miette!("strengthening work should be queued"))?;
    assert_eq!(
        strengthen.work_id(),
        "assessment_work.recommendation:assess-belvedere-ingestion-1"
    );
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
    assert_eq!(missing_assessment_gap.queue_lane(), "assessment");
    assert_eq!(missing_assessment_gap.action(), "assess proof coverage");
    assert!(!missing_assessment_gap.high_priority());

    let missing_proof_gap = model
        .gaps()
        .iter()
        .find(|gap| gap.kind() == LandscapeGapKind::MissingProof)
        .ok_or_else(|| miette!("missing-proof gap should be visible"))?;
    assert_eq!(missing_proof_gap.severity(), GapSeverity::Blocking);
    assert_eq!(missing_proof_gap.queue_lane(), "proof_coverage");
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
    assert_eq!(witness_gap.queue_lane(), "witness_review");
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
    assert_eq!(
        raw_root.source_record_ids(),
        raw_root.trace().source_record_ids()
    );
    assert!(
        raw_root
            .source_record_ids()
            .contains(&"evidence_link:belvedere::ingestion::DecodedRegistrySnapshot#0".to_owned())
    );

    let proof_root = model
        .provenance_roots()
        .iter()
        .find(|root| root.claim_id() == "belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| miette!("Kani proof root should be cataloged"))?;
    assert_eq!(proof_root.root_kind(), "proof_registration_root");
    assert_eq!(proof_root.verifier_count(), 1);
    assert_eq!(proof_root.assessment_count(), 1);
    assert_eq!(proof_root.gap_count(), 0);
    assert!(
        proof_root
            .source_record_ids()
            .contains(&"kani_proof:belvedere::ingestion::record_identity_is_preserved@belvedere:verify_record_identity_is_preserved".to_owned())
    );
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
    assert_eq!(model.overview().current_assessment_count(), 1);
    assert_eq!(model.overview().superseded_assessment_count(), 1);
    assert!(current.current());
    assert_eq!(current.superseded_count(), 1);
    assert_eq!(current.superseded_by(), None);

    let history = model
        .assessment_history()
        .iter()
        .find(|item| item.proof_id() == "belvedere::ingestion::record_identity_is_preserved")
        .ok_or_else(|| miette!("stale fixture should expose grouped assessment history"))?;
    assert_eq!(history.current_assessment_id(), "assess-current");
    assert_eq!(
        history.assessment_ids(),
        &vec!["assess-old".to_owned(), "assess-current".to_owned()]
    );
    assert_eq!(
        history.recommendation_flow(),
        &vec!["Strengthen".to_owned(), "Accept".to_owned()]
    );
    assert_eq!(history.history_count(), 2);
    assert_eq!(history.superseded_count(), 1);
    assert_eq!(history.weak_dimensions(), &Vec::<String>::new());
    assert_eq!(history.current_resolution_label(), "keep current proof");
    assert!(
        history
            .current_comment()
            .contains("Latest review supersedes the earlier strengthening request")
    );

    let older = model
        .assessment_queue()
        .iter()
        .find(|item| item.assessment_id() == "assess-old")
        .ok_or_else(|| miette!("older assessment should be queued"))?;
    assert!(!older.current());
    assert_eq!(older.superseded_count(), 0);
    assert_eq!(older.superseded_by(), Some("assess-current"));
    Ok(())
}

fn fixture_input() -> Result<ViewModelInput> {
    fixture_input_with_assessments(include_bytes!("fixtures/proof_assessments.jsonl").to_vec())
}

fn fixture_input_with_assessments(assessment_bytes: Vec<u8>) -> Result<ViewModelInput> {
    fixture_input_with_artifacts(
        include_bytes!("fixtures/registry_dump.json").to_vec(),
        assessment_bytes,
    )
}

fn fixture_input_with_artifacts(
    registry_bytes: Vec<u8>,
    assessment_bytes: Vec<u8>,
) -> Result<ViewModelInput> {
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
    let landscape_input = LandscapeInput::from_ingestion(
        registry,
        &registry_receipt,
        assessments,
        &assessment_receipt,
    );
    let (landscape, landscape_receipt) = LandscapeModeling
        .model(landscape_input)
        .into_diagnostic()
        .wrap_err("landscape modeling is conservative and infallible")?;

    Ok(ViewModelInput::from_landscape(
        landscape,
        &landscape_receipt,
    ))
}

fn assert_view_model_receipt_identity(receipt: &belvedere::ViewModelReceipt) {
    assert!(
        receipt
            .receipt_id()
            .starts_with("belvedere.receipt.view_model:")
    );
    assert_eq!(
        receipt.receipt_id(),
        &format!("belvedere.receipt.view_model:{}", receipt.receipt_digest())
    );
}

fn is_hex_digest(digest: &str) -> bool {
    digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
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
