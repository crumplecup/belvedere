//! Receipts for auditor navigation view-model exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{hex_digest, update_field, update_fields};

const VIEW_MODEL_RECEIPT_SCHEMA_ID: &str = "belvedere.receipts.view_model.v1";
const VIEW_MODEL_RECEIPT_ID_PREFIX: &str = "belvedere.receipt.view_model";

/// Audit receipt emitted by a view-model exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ViewModelReceipt {
    contract: String,
    #[getter(skip)]
    counts: ViewModelReceiptCounts,
    receipt_id: String,
    receipt_digest: String,
    lane_counts: BTreeMap<String, usize>,
    facet_group_counts: BTreeMap<String, usize>,
    source_artifact_counts: BTreeMap<String, usize>,
    source_kind_counts: BTreeMap<String, usize>,
    proof_chain_topology_counts: BTreeMap<String, usize>,
    provenance_root_kind_counts: BTreeMap<String, usize>,
    review_workflow_counts: BTreeMap<String, usize>,
    trace_kind_counts: BTreeMap<String, usize>,
    trace_source_counts: BTreeMap<String, usize>,
    item_kind_counts: BTreeMap<String, usize>,
    source_landscape_receipt_id: String,
    view_item_ids: Vec<String>,
    source_record_ids: Vec<String>,
    warnings: Vec<String>,
}

impl ViewModelReceipt {
    /// Modeled view counts.
    #[must_use]
    pub const fn counts(&self) -> ViewModelReceiptCounts {
        self.counts
    }
}

/// Evidence used to identify a view-model receipt.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(vis = "pub(crate)")]
pub(crate) struct ViewModelReceiptDigestEvidence<'evidence> {
    /// Exchange contract that emitted the receipt.
    contract: &'evidence str,
    /// Counts for emitted auditor navigation evidence.
    counts: ViewModelReceiptCounts,
    /// Counts grouped by review lane.
    lane_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by filter facet group.
    facet_group_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by source artifact kind.
    source_artifact_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by source kind.
    source_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by proof-chain topology.
    proof_chain_topology_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by provenance root kind.
    provenance_root_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by review workflow state.
    review_workflow_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by trace item kind.
    trace_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by trace source.
    trace_source_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by emitted view-model item kind.
    item_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Upstream landscape receipt used as input evidence.
    source_landscape_receipt_id: &'evidence str,
    /// Stable view-model item IDs emitted by the exchange.
    view_item_ids: &'evidence [String],
    /// Source record IDs represented by the view model.
    source_record_ids: &'evidence [String],
    /// Recoverable warnings emitted by the exchange.
    warnings: &'evidence [String],
}

/// Build the stable identifier and digest for a view-model receipt.
#[instrument(level = "debug", skip(evidence))]
pub(crate) fn view_model_receipt_identity(
    evidence: ViewModelReceiptDigestEvidence<'_>,
) -> (String, String) {
    let receipt_digest = view_model_receipt_digest(evidence);
    let receipt_id = format!("{VIEW_MODEL_RECEIPT_ID_PREFIX}:{receipt_digest}");
    (receipt_id, receipt_digest)
}

/// Counts emitted by an auditor navigation receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct ViewModelReceiptCounts {
    claim_cards: usize,
    filter_facets: usize,
    backend_coverage_items: usize,
    proof_chain_items: usize,
    review_workflow_items: usize,
    assessment_history_items: usize,
    assessment_items: usize,
    assessment_work_items: usize,
    witness_items: usize,
    root_items: usize,
    gap_items: usize,
    source_items: usize,
    claim_detail_items: usize,
}

impl ViewModelReceiptCounts {
    /// Number of claim explorer cards.
    #[must_use]
    pub const fn claim_cards(self) -> usize {
        self.claim_cards
    }

    /// Number of auditor filter facets.
    #[must_use]
    pub const fn filter_facets(self) -> usize {
        self.filter_facets
    }

    /// Number of backend coverage status entries.
    #[must_use]
    pub const fn backend_coverage_items(self) -> usize {
        self.backend_coverage_items
    }

    /// Number of proof-chain navigation items.
    #[must_use]
    pub const fn proof_chain_items(self) -> usize {
        self.proof_chain_items
    }

    /// Number of review workflow tasks.
    #[must_use]
    pub const fn review_workflow_items(self) -> usize {
        self.review_workflow_items
    }

    /// Number of proof-grouped assessment history items.
    #[must_use]
    pub const fn assessment_history_items(self) -> usize {
        self.assessment_history_items
    }

    /// Number of assessment queue items.
    #[must_use]
    pub const fn assessment_items(self) -> usize {
        self.assessment_items
    }

    /// Number of assessment work items.
    #[must_use]
    pub const fn assessment_work_items(self) -> usize {
        self.assessment_work_items
    }

    /// Number of witness surface items.
    #[must_use]
    pub const fn witness_items(self) -> usize {
        self.witness_items
    }

    /// Number of provenance root items.
    #[must_use]
    pub const fn root_items(self) -> usize {
        self.root_items
    }

    /// Number of gap items.
    #[must_use]
    pub const fn gap_items(self) -> usize {
        self.gap_items
    }

    /// Number of source navigation items.
    #[must_use]
    pub const fn source_items(self) -> usize {
        self.source_items
    }

    /// Number of claim detail deep-dive items.
    #[must_use]
    pub const fn claim_detail_items(self) -> usize {
        self.claim_detail_items
    }
}

#[instrument(level = "debug", skip(evidence))]
fn view_model_receipt_digest(evidence: ViewModelReceiptDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, VIEW_MODEL_RECEIPT_SCHEMA_ID);
    update_field(&mut hasher, evidence.contract);
    update_field(&mut hasher, &evidence.counts.claim_cards().to_string());
    update_field(
        &mut hasher,
        &evidence.counts.claim_detail_items().to_string(),
    );
    update_field(&mut hasher, &evidence.counts.filter_facets().to_string());
    update_field(
        &mut hasher,
        &evidence.counts.backend_coverage_items().to_string(),
    );
    update_field(
        &mut hasher,
        &evidence.counts.proof_chain_items().to_string(),
    );
    update_field(
        &mut hasher,
        &evidence.counts.review_workflow_items().to_string(),
    );
    update_field(
        &mut hasher,
        &evidence.counts.assessment_history_items().to_string(),
    );
    update_field(&mut hasher, &evidence.counts.assessment_items().to_string());
    update_field(
        &mut hasher,
        &evidence.counts.assessment_work_items().to_string(),
    );
    update_field(&mut hasher, &evidence.counts.witness_items().to_string());
    update_field(&mut hasher, &evidence.counts.root_items().to_string());
    update_field(&mut hasher, &evidence.counts.gap_items().to_string());
    update_field(&mut hasher, &evidence.counts.source_items().to_string());
    update_counts(&mut hasher, evidence.lane_counts);
    update_counts(&mut hasher, evidence.facet_group_counts);
    update_counts(&mut hasher, evidence.source_artifact_counts);
    update_counts(&mut hasher, evidence.source_kind_counts);
    update_counts(&mut hasher, evidence.proof_chain_topology_counts);
    update_counts(&mut hasher, evidence.provenance_root_kind_counts);
    update_counts(&mut hasher, evidence.review_workflow_counts);
    update_counts(&mut hasher, evidence.trace_kind_counts);
    update_counts(&mut hasher, evidence.trace_source_counts);
    update_counts(&mut hasher, evidence.item_kind_counts);
    update_field(&mut hasher, evidence.source_landscape_receipt_id);
    update_fields(&mut hasher, evidence.view_item_ids);
    update_fields(&mut hasher, evidence.source_record_ids);
    update_fields(&mut hasher, evidence.warnings);
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(hasher, counts))]
fn update_counts(hasher: &mut Sha256, counts: &BTreeMap<String, usize>) {
    for (name, count) in counts {
        update_field(hasher, name);
        update_field(hasher, &count.to_string());
    }
}
