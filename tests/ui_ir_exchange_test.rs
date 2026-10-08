use std::collections::BTreeMap;

use belvedere::{
    ArtifactKind, AssessmentStreamIngestion, AuditorNavigationProjection, ElicitUiProjection,
    LandscapeInput, LandscapeModeling, RawAmenableArtifact, RegistryDumpIngestion,
    UI_IR_SECTION_ASSESSMENT_HISTORY, UI_IR_SECTION_BACKEND_COVERAGE, UI_IR_SECTION_CLAIM_DETAIL,
    UI_IR_SECTION_CLAIM_EXPLORER, UI_IR_SECTION_FILTERS, UI_IR_SECTION_OVERVIEW,
    UI_IR_SECTION_PROOF_CHAINS, UI_IR_SECTION_REVIEW_WORKFLOW, UI_IR_SECTION_SOURCE_NAVIGATION,
    UiIrInput, ViewModelInput,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};

#[test]
fn ui_ir_projection_mints_elicit_ui_standards_and_tree_proofs() -> Result<()> {
    belvedere::init_tracing();

    let (projection, receipt) = ElicitUiProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("fixture view model should project into elicit_ui")?;

    assert_eq!(projection.sections().len(), 13);
    assert!(
        projection
            .sections()
            .iter()
            .any(|section| section.id() == UI_IR_SECTION_OVERVIEW && section.name() == "overview")
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_CLAIM_EXPLORER.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_CLAIM_DETAIL.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_FILTERS.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_BACKEND_COVERAGE.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_PROOF_CHAINS.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_REVIEW_WORKFLOW.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_ASSESSMENT_HISTORY.to_owned())
    );
    assert!(
        receipt
            .section_ids()
            .contains(&UI_IR_SECTION_SOURCE_NAVIGATION.to_owned())
    );
    assert_eq!(
        receipt.section_root_widget_ids(),
        &projection
            .sections()
            .iter()
            .map(belvedere::UiIrSection::root_widget_id)
            .collect::<Vec<_>>()
    );
    assert_eq!(
        receipt.trace_widget_ids(),
        &projection
            .trace_widgets()
            .iter()
            .map(belvedere::TraceWidgetRef::widget_id)
            .collect::<Vec<_>>()
    );
    let trace_widget_ref_ids = projection
        .trace_widgets()
        .iter()
        .map(|widget| widget.trace_ref_id().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(receipt.trace_widget_ref_ids(), &trace_widget_ref_ids);
    let mut unique_trace_widget_ref_ids = trace_widget_ref_ids.clone();
    unique_trace_widget_ref_ids.sort();
    unique_trace_widget_ref_ids.dedup();
    assert_eq!(
        unique_trace_widget_ref_ids.len(),
        trace_widget_ref_ids.len()
    );
    let section_ids = projection
        .sections()
        .iter()
        .map(|section| section.id().as_str())
        .collect::<Vec<_>>();
    assert!(
        projection
            .trace_widgets()
            .iter()
            .all(|widget| section_ids.contains(&widget.section_id().as_str()))
    );
    assert!(
        projection
            .trace_widgets()
            .iter()
            .any(
                |widget| widget.section_id() == UI_IR_SECTION_OVERVIEW && widget.role() == "region"
            )
    );
    assert!(
        projection
            .minted_standards()
            .iter()
            .any(|mint| mint.prop() == "elicit_ui::WcagVerified")
    );
    assert!(
        receipt
            .minted_standards()
            .iter()
            .any(|standard| standard.ends_with("ElicitUiIrSourcedStandard"))
    );
    assert_eq!(
        receipt
            .minted_prop_counts()
            .get("minted_elicit_ui_prop_IrSourced"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .minted_prop_counts()
            .get("minted_elicit_ui_prop_WcagVerified"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .minted_prop_counts()
            .get("minted_elicit_ui_prop_RenderComplete"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .minted_prop_counts()
            .get("minted_elicit_ui_prop_RolePreserved"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .proof_token_counts()
            .get("ui_ir_proof_token_ir_sourced"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .proof_token_counts()
            .get("ui_ir_proof_token_wcag_verified"),
        Some(&1)
    );
    assert_eq!(
        projection
            .viewport_verifications()
            .iter()
            .map(|verification| verification.viewport_id())
            .collect::<Vec<_>>(),
        vec!["390x844", "768x1024", "1440x960"]
    );
    assert_eq!(
        receipt.viewport_ids(),
        &projection
            .viewport_verifications()
            .iter()
            .map(|verification| verification.viewport_id())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        receipt
            .proof_token_counts()
            .get("ui_ir_proof_token_viewport_wcag_verified"),
        Some(&3)
    );
    assert_eq!(
        receipt
            .viewport_verification_counts()
            .get("ui_ir_viewports_verified_total"),
        Some(&3)
    );
    for verification in projection.viewport_verifications() {
        assert!(verification.node_count() >= projection.trace_widgets().len());
        assert_eq!(
            receipt.viewport_verification_counts().get(&format!(
                "ui_ir_viewport_nodes:{}",
                verification.viewport_id()
            )),
            Some(&verification.node_count())
        );
    }
    assert!(receipt.counts().node_count() >= projection.trace_widgets().len());
    assert_eq!(receipt.role_counts().get("role_button"), Some(&27));
    assert_eq!(receipt.role_counts().get("role_checkbox"), Some(&10));
    assert_eq!(receipt.role_counts().get("role_link"), Some(&10));
    assert_eq!(receipt.role_counts().get("role_region"), Some(&1));
    assert_eq!(receipt.role_counts().get("role_status"), Some(&12));
    assert_eq!(
        receipt
            .section_widget_counts()
            .get(UI_IR_SECTION_CLAIM_EXPLORER),
        Some(&4)
    );
    assert_eq!(
        receipt
            .section_widget_counts()
            .get(UI_IR_SECTION_CLAIM_DETAIL),
        Some(&4)
    );
    assert_eq!(
        receipt
            .section_widget_counts()
            .get(UI_IR_SECTION_SOURCE_NAVIGATION),
        Some(&7)
    );
    assert_eq!(
        receipt
            .section_widget_counts()
            .get(UI_IR_SECTION_REVIEW_WORKFLOW),
        Some(&4)
    );
    assert_eq!(
        receipt
            .section_widget_counts()
            .get(UI_IR_SECTION_ASSESSMENT_HISTORY),
        Some(&2)
    );
    assert_eq!(
        receipt
            .section_population_counts()
            .get("ui_ir_sections_populated"),
        Some(&13)
    );
    assert_eq!(
        receipt
            .section_population_counts()
            .get("ui_ir_sections_empty"),
        Some(&0)
    );
    assert_eq!(
        receipt
            .section_population_counts()
            .get("ui_ir_section_widgets_total"),
        Some(&projection.trace_widgets().len())
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_sections_with_source_widgets"),
        Some(&13)
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_sections_without_source_widgets"),
        Some(&0)
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_section_trace_widgets:belvedere.ui.section.overview"),
        Some(&1)
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_section_trace_source_links:belvedere.ui.section.overview"),
        Some(&7)
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_section_trace_widgets:belvedere.ui.section.source_navigation"),
        Some(&7)
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_section_trace_source_links:belvedere.ui.section.source_navigation"),
        Some(&7)
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_section_unassigned_trace_widgets"),
        Some(&0)
    );
    let trace_widgets_with_sources = projection
        .trace_widgets()
        .iter()
        .filter(|widget| !widget.source_record_ids().is_empty())
        .count();
    assert_eq!(
        receipt
            .trace_source_counts()
            .get("trace_widgets_with_sources"),
        Some(&trace_widgets_with_sources)
    );
    assert_eq!(
        receipt
            .trace_source_counts()
            .get("trace_unique_source_records"),
        Some(&7)
    );
    assert_eq!(
        receipt.trace_source_counts().get("trace_source_links"),
        Some(&receipt.counts().trace_ref_count())
    );
    assert_eq!(
        receipt.trace_source_kind_counts(),
        &trace_source_kind_counts(projection.trace_widgets())
    );
    assert!(
        receipt
            .trace_source_kind_counts()
            .get("trace_source_kind_assessment")
            .is_some_and(|count| *count > 0)
    );
    assert!(
        receipt
            .trace_source_kind_counts()
            .get("trace_source_kind_proof_record")
            .is_some_and(|count| *count > 0)
    );
    assert_eq!(
        receipt
            .section_trace_counts()
            .get("ui_ir_section_trace_source_links_total"),
        receipt.trace_source_counts().get("trace_source_links")
    );
    assert!(projection.trace_widgets().iter().any(|widget| {
        widget.role() == "region"
            && widget.label()
            == "Overview: 4 claims, 2 proofs, 2 assessments (2 current, 0 superseded), 4 gaps, 3 high-priority gaps"
    }));

    let _ir_sourced = projection.proofs().ir_sourced();
    let _wcag_verified = projection.proofs().wcag_verified();
    Ok(())
}

fn trace_source_kind_counts(
    trace_widgets: &[belvedere::TraceWidgetRef],
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for source_record_id in trace_widgets
        .iter()
        .flat_map(belvedere::TraceWidgetRef::source_record_ids)
    {
        let kind = source_record_id
            .split_once(':')
            .map_or(source_record_id.as_str(), |(kind, _identity)| kind);
        *counts
            .entry(format!("trace_source_kind_{kind}"))
            .or_default() += 1;
    }
    counts
}

#[test]
fn ui_ir_projection_preserves_trace_widgets_for_auditor_deep_dives() -> Result<()> {
    belvedere::init_tracing();

    let (projection, receipt) = ElicitUiProjection
        .project(fixture_input()?)
        .into_diagnostic()
        .wrap_err("fixture view model should project into elicit_ui")?;

    let decoded_claim = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && widget
                    .label()
                    .starts_with("Claim belvedere::ingestion::DecodedRegistrySnapshot")
        })
        .ok_or_else(|| miette!("claim explorer item should have a traceable widget"))?;
    assert_eq!(decoded_claim.section_id(), UI_IR_SECTION_CLAIM_EXPLORER);
    assert_eq!(
        decoded_claim.trace_ref_id(),
        "elicit_ui.trace_widget_ref:belvedere.ui.section.claim_explorer:button:belvedere::ingestion::DecodedRegistrySnapshot"
    );
    assert!(decoded_claim.label().contains("derived claim"));
    assert!(decoded_claim.label().contains("1 direct basis"));
    assert!(decoded_claim.label().contains("1 verifier proof"));
    assert!(decoded_claim.label().contains("1 contract"));
    assert!(decoded_claim.label().contains("0 assessments"));
    assert!(decoded_claim.label().contains("2 gaps"));
    assert!(
        decoded_claim
            .label()
            .contains("belvedere.filter.verifier:kani")
    );
    assert!(
        decoded_claim
            .label()
            .contains("belvedere.filter.priority:high")
    );
    assert!(
        decoded_claim.source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );
    let decoded_detail = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.claim_id() == "belvedere::ingestion::DecodedRegistrySnapshot"
                && widget
                    .label()
                    .starts_with("Claim detail belvedere::ingestion::DecodedRegistrySnapshot")
        })
        .ok_or_else(|| miette!("claim detail item should have a traceable widget"))?;
    assert_eq!(decoded_detail.section_id(), UI_IR_SECTION_CLAIM_DETAIL);
    assert!(decoded_detail.label().contains("derived claim"));
    assert!(
        decoded_detail
            .label()
            .contains("direct bases belvedere::ingestion::RawAmenableArtifact")
    );
    assert!(
        decoded_detail
            .label()
            .contains("resolved roots belvedere::ingestion::RawAmenableArtifact")
    );
    assert!(decoded_detail.label().contains("verifier proofs kani"));
    assert!(
        decoded_detail
            .label()
            .contains("proof statuses no proof execution statuses")
    );
    assert!(decoded_detail.label().contains("kani ensures"));
    assert!(
        decoded_detail
            .label()
            .contains("MissingAssessment review: assess proof coverage")
    );
    assert!(
        decoded_detail.label().contains(
            "review state elevated; next action: review trusted or opaque witness support"
        )
    );
    assert!(decoded_detail.label().contains("4 source records"));
    assert_eq!(
        decoded_detail.source_record_ids(),
        decoded_claim.source_record_ids()
    );
    let proof_detail = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.claim_id() == "belvedere::ingestion::record_identity_is_preserved"
                && widget
                    .label()
                    .starts_with("Claim detail belvedere::ingestion::record_identity_is_preserved")
        })
        .ok_or_else(|| miette!("proof claim detail item should have a traceable widget"))?;
    assert!(
        proof_detail
            .label()
            .contains("status verified scope bounded harness")
    );

    let gap_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label().starts_with(
                "Gap MissingAssessment for belvedere::ingestion::DecodedRegistrySnapshot in assessment lane is review priority; next action: assess proof coverage",
            )
        })
        .ok_or_else(|| miette!("gap queue should have traceable widgets"))?;
    assert_eq!(gap_widget.section_id(), belvedere::UI_IR_SECTION_GAP_QUEUE);
    assert!(!gap_widget.source_record_ids().is_empty());
    let filter_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| widget.label().starts_with("Filter Verifier kani"))
        .ok_or_else(|| miette!("filter facets should have traceable widgets"))?;
    assert_eq!(filter_widget.claim_id(), "belvedere.filter.verifier:kani");
    assert_filter_widget(
        &projection,
        "Filter Claim family belvedere::ingestion selects 4 items",
        "belvedere.filter.claim_family:belvedere::ingestion",
    )?;
    assert_filter_widget(
        &projection,
        "Filter Support kind checked_with_trusted selects 1 items",
        "belvedere.filter.support_kind:checked_with_trusted",
    )?;
    assert_filter_widget(
        &projection,
        "Filter Package belvedere selects 1 items",
        "belvedere.filter.package:belvedere",
    )?;
    assert_filter_widget(
        &projection,
        "Filter Priority high selects 3 items",
        "belvedere.filter.priority:high",
    )?;
    let coverage_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label().starts_with(
                "Backend coverage creusot for belvedere::ingestion::DecodedRegistrySnapshot is missing",
            )
        })
        .ok_or_else(|| miette!("backend coverage should have traceable widgets"))?;
    assert_eq!(
        coverage_widget.claim_id(),
        "belvedere::ingestion::DecodedRegistrySnapshot"
    );
    let chain_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget
                .label()
                .starts_with("Proof chain belvedere::ingestion::DecodedRegistrySnapshot")
        })
        .ok_or_else(|| miette!("proof-chain view should have traceable widgets"))?;
    assert!(
        chain_widget
            .label()
            .contains("roots belvedere::ingestion::RawAmenableArtifact")
    );
    assert!(chain_widget.label().contains("is elevated"));
    assert!(
        chain_widget
            .label()
            .contains("next action: review trusted or opaque witness support")
    );
    assert!(
        chain_widget
            .label()
            .contains("direct bases belvedere::ingestion::RawAmenableArtifact")
    );
    assert!(chain_widget.label().contains("verifier proofs kani"));
    assert!(
        chain_widget
            .label()
            .contains("proof statuses no proof execution statuses")
    );
    assert!(
        chain_widget
            .label()
            .contains("chain backend coverage kani incomplete (1 covered, 1 missing)")
    );
    assert!(
        chain_widget
            .label()
            .contains("creusot incomplete (0 covered, 2 missing)")
    );
    assert!(chain_widget.label().contains("2 linked claims"));
    assert!(chain_widget.label().contains("1 level deep"));
    assert!(chain_widget.label().contains("0 branching claims"));
    assert!(chain_widget.label().contains("3 chain gaps"));
    assert!(chain_widget.label().contains("2 gaps"));
    assert!(!chain_widget.source_record_ids().is_empty());
    let proof_chain_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.claim_id() == "belvedere::ingestion::record_identity_is_preserved"
                && widget
                    .label()
                    .starts_with("Proof chain belvedere::ingestion::record_identity_is_preserved")
        })
        .ok_or_else(|| miette!("proof claim chain should have a traceable widget"))?;
    assert!(
        proof_chain_widget
            .label()
            .contains("status verified scope bounded harness")
    );
    let workflow_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label().starts_with(
                "Review task belvedere.review:belvedere::ingestion::DecodedRegistrySnapshot:elevated:witness-review",
            )
        })
        .ok_or_else(|| miette!("review workflow task should have a traceable widget"))?;
    assert!(
        workflow_widget
            .label()
            .contains("lane elevated:witness-review priority elevated")
    );
    assert!(workflow_widget.label().contains(
        "linked gaps view_model.gap:belvedere::ingestion::DecodedRegistrySnapshot:missing_assessment, view_model.gap:belvedere::ingestion::DecodedRegistrySnapshot:trusted_or_opaque_witness_support"
    ));
    assert!(
        workflow_widget
            .label()
            .contains("action: review trusted or opaque witness support")
    );
    assert!(
        workflow_widget
            .label()
            .contains("rationale: proof chain is elevated with 2 modeled gaps")
    );
    assert!(
        workflow_widget.source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );
    let witness_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label().starts_with(
                "Witness kani checked_with_trusted for belvedere::ingestion::DecodedRegistrySnapshot",
            )
        })
        .ok_or_else(|| miette!("witness support distribution should be traceable"))?;
    assert!(
        witness_widget
            .label()
            .contains("0 trivial, 1 checked, 1 trusted, 0 opaque supports")
    );
    let assessment_work_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label().starts_with(
                "Assessment work assess proof coverage lane assessment for belvedere::ingestion::DecodedRegistrySnapshot",
            )
        })
        .ok_or_else(|| miette!("assessment work should have traceable widgets"))?;
    assert!(
        assessment_work_widget
            .label()
            .contains("resolution: collect proof assessment")
    );
    assert!(
        assessment_work_widget.source_record_ids().contains(
            &"proof_record:belvedere::ingestion::DecodedRegistrySnapshot@kani".to_owned()
        )
    );
    assert!(projection.trace_widgets().iter().any(|widget| {
        widget.label().starts_with(
            "Assessment work strengthen proof lane strengthening for belvedere::ingestion::record_identity_is_preserved",
        ) && widget
            .label()
            .contains("resolution: strengthen current proof")
    }));
    let assessment_history_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label().starts_with(
                "Assessment history for belvedere::ingestion::record_identity_is_preserved current assess-belvedere-ingestion-1",
            )
        })
        .ok_or_else(|| miette!("assessment history should have a traceable widget"))?;
    assert!(
        assessment_history_widget
            .label()
            .contains("recommendations Strengthen")
    );
    assert!(
        assessment_history_widget
            .label()
            .contains("weak dimensions assertion_strength, adversarial_coverage")
    );
    assert!(
        assessment_history_widget
            .label()
            .contains("resolution: strengthen current proof")
    );
    assert_eq!(assessment_history_widget.source_record_ids().len(), 2);
    let root_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label().starts_with(
                "Provenance root belvedere::ingestion::RawAmenableArtifact is unmodeled_root",
            )
        })
        .ok_or_else(|| miette!("provenance root catalog should have traceable widgets"))?;
    assert!(
        root_widget
            .label()
            .contains("verifier proofs no verifier proofs")
    );
    assert!(root_widget.label().contains("assessments no assessments"));
    assert!(
        root_widget
            .label()
            .contains("gaps MissingProof blocking: establish modeled proof coverage")
    );
    assert!(
        root_widget
            .label()
            .contains("next action: establish modeled proof coverage")
    );
    assert!(root_widget.label().contains("1 gap"));
    assert!(root_widget.label().contains("7 source records"));
    let source_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget
                .label()
                .starts_with("Proof record belvedere::ingestion")
        })
        .ok_or_else(|| miette!("source navigation should have traceable widgets"))?;
    assert!(
        source_widget
            .label()
            .contains("opens registry_dump proof_record target")
    );
    assert_eq!(source_widget.source_record_ids().len(), 1);
    let proof_source_widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| {
            widget.label()
                == "Kani proof belvedere::ingestion::record_identity_is_preserved opens registry_dump kani_proof target belvedere::ingestion::record_identity_is_preserved"
        })
        .ok_or_else(|| miette!("Kani proof source navigation should be projected"))?;
    assert_eq!(proof_source_widget.source_record_ids().len(), 1);
    assert!(projection.trace_widgets().iter().any(|widget| {
        widget.label()
            == "Assessment assess-belvedere-ingestion-1 opens proof_assessments assessment target assess-belvedere-ingestion-1"
    }));
    let preserved_trace_refs = projection
        .trace_widgets()
        .iter()
        .map(|widget| widget.source_record_ids().len())
        .sum::<usize>();
    assert_eq!(receipt.counts().trace_ref_count(), preserved_trace_refs);
    Ok(())
}

#[test]
fn ui_ir_projection_labels_assessment_recency() -> Result<()> {
    belvedere::init_tracing();

    let (projection, _receipt) = ElicitUiProjection
        .project(fixture_input_with_assessments(
            STALE_ASSESSMENTS.as_bytes().to_vec(),
        )?)
        .into_diagnostic()
        .wrap_err("fixture view model should project recency into elicit_ui")?;

    assert!(projection.trace_widgets().iter().any(|widget| {
        widget
            .label()
            .starts_with("Assessment assess-current (current, supersedes 1 older assessment)")
            && widget.label().contains("lane accepted")
            && widget.label().contains("resolution: keep current proof")
            && widget.label().contains("no weak dimensions")
    }));
    assert!(projection.trace_widgets().iter().any(|widget| {
        widget
            .label()
            .starts_with("Assessment assess-old (superseded by assess-current)")
            && widget.label().contains("lane strengthening")
            && widget
                .label()
                .contains("resolution: strengthen current proof")
            && widget
                .label()
                .contains("weak dimensions assertion_strength, adversarial_coverage")
    }));
    assert!(projection.trace_widgets().iter().any(|widget| {
        widget.label().starts_with(
            "Assessment history for belvedere::ingestion::record_identity_is_preserved current assess-current",
        ) && widget
            .label()
            .contains("has assess-old, assess-current")
            && widget
                .label()
                .contains("recommendations Strengthen, Accept")
            && widget.label().contains("1 superseded assessment")
    }));
    Ok(())
}

fn fixture_input() -> Result<UiIrInput> {
    fixture_input_with_assessments(include_bytes!("fixtures/proof_assessments.jsonl").to_vec())
}

fn fixture_input_with_assessments(assessment_bytes: Vec<u8>) -> Result<UiIrInput> {
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
    let (model, _view_receipt) = AuditorNavigationProjection
        .project(ViewModelInput::new(landscape))
        .into_diagnostic()
        .wrap_err("view-model projection is trace-preserving and infallible")?;

    Ok(UiIrInput::new(model))
}

fn assert_filter_widget(
    projection: &belvedere::UiIrProjection,
    label: &str,
    claim_id: &str,
) -> Result<()> {
    let widget = projection
        .trace_widgets()
        .iter()
        .find(|widget| widget.label() == label)
        .ok_or_else(|| miette!("{label} should have a traceable filter widget"))?;
    assert_eq!(widget.claim_id(), claim_id);
    assert!(!widget.source_record_ids().is_empty());
    Ok(())
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
