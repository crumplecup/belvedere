//! Receipts for UI IR projection exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{hex_digest, update_field, update_fields};

const UI_IR_RECEIPT_SCHEMA_ID: &str = "belvedere.receipts.ui_ir.v1";
const UI_IR_RECEIPT_ID_PREFIX: &str = "belvedere.receipt.ui_ir";

/// Audit receipt emitted by an `elicit_ui` projection exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct UiIrReceipt {
    contract: String,
    #[getter(skip)]
    counts: UiIrReceiptCounts,
    receipt_id: String,
    receipt_digest: String,
    source_view_model_receipt_id: String,
    source_view_model_receipt_digest: String,
    source_record_ids: Vec<String>,
    section_ids: Vec<String>,
    section_root_widget_ids: Vec<u64>,
    trace_widget_ids: Vec<u64>,
    trace_widget_ref_ids: Vec<String>,
    trace_widget_digests: Vec<String>,
    output_ids: Vec<String>,
    output_kind_counts: BTreeMap<String, usize>,
    role_counts: BTreeMap<String, usize>,
    section_widget_counts: BTreeMap<String, usize>,
    section_population_counts: BTreeMap<String, usize>,
    section_trace_counts: BTreeMap<String, usize>,
    viewport_ids: Vec<String>,
    viewport_verification_counts: BTreeMap<String, usize>,
    trace_source_counts: BTreeMap<String, usize>,
    trace_source_kind_counts: BTreeMap<String, usize>,
    source_locator_counts: BTreeMap<String, usize>,
    minted_prop_counts: BTreeMap<String, usize>,
    proof_token_counts: BTreeMap<String, usize>,
    minted_standards: Vec<String>,
}

impl UiIrReceipt {
    /// UI IR projection counts.
    #[must_use]
    pub const fn counts(&self) -> UiIrReceiptCounts {
        self.counts
    }
}

/// Evidence used to identify a UI IR projection receipt.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(vis = "pub(crate)")]
pub(crate) struct UiIrReceiptDigestEvidence<'evidence> {
    /// Exchange contract that emitted the receipt.
    contract: &'evidence str,
    /// Counts for emitted `elicit_ui` projection evidence.
    counts: UiIrReceiptCounts,
    /// Upstream view-model receipt used as input evidence.
    source_view_model_receipt_id: &'evidence str,
    /// Upstream view-model receipt digest used as input evidence.
    source_view_model_receipt_digest: &'evidence str,
    /// Source record IDs represented by the UI IR projection.
    source_record_ids: &'evidence [String],
    /// Stable section IDs emitted by the projection.
    section_ids: &'evidence [String],
    /// Root widget IDs for emitted sections.
    section_root_widget_ids: &'evidence [u64],
    /// Traceable widget IDs emitted by the projection.
    trace_widget_ids: &'evidence [u64],
    /// Stable trace widget reference IDs emitted by the projection.
    trace_widget_ref_ids: &'evidence [String],
    /// Stable trace widget digests emitted by the projection.
    trace_widget_digests: &'evidence [String],
    /// Stable output IDs emitted by the projection.
    output_ids: &'evidence [String],
    /// Counts grouped by UI IR output kind.
    output_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by `elicit_ui` role.
    role_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by section widget population.
    section_widget_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by section population status.
    section_population_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by section source trace coverage.
    section_trace_counts: &'evidence BTreeMap<String, usize>,
    /// Verified viewport IDs.
    viewport_ids: &'evidence [String],
    /// Counts grouped by viewport verification result.
    viewport_verification_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by source trace preservation.
    trace_source_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by source record kind in traces.
    trace_source_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by source navigation locator preservation.
    source_locator_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by minted `elicit_ui` prop.
    minted_prop_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by UI proof token kind.
    proof_token_counts: &'evidence BTreeMap<String, usize>,
    /// Stable minted standard IDs emitted by the projection.
    minted_standards: &'evidence [String],
}

/// Build the stable identifier and digest for a UI IR projection receipt.
#[instrument(level = "debug", skip(evidence))]
pub(crate) fn ui_ir_receipt_identity(evidence: UiIrReceiptDigestEvidence<'_>) -> (String, String) {
    let receipt_digest = ui_ir_receipt_digest(evidence);
    let receipt_id = format!("{UI_IR_RECEIPT_ID_PREFIX}:{receipt_digest}");
    (receipt_id, receipt_digest)
}

/// Counts emitted by a UI IR projection receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct UiIrReceiptCounts {
    section_count: usize,
    widget_count: usize,
    trace_ref_count: usize,
    node_count: usize,
}

impl UiIrReceiptCounts {
    /// Build UI IR projection counts.
    #[must_use]
    pub const fn new(
        section_count: usize,
        widget_count: usize,
        trace_ref_count: usize,
        node_count: usize,
    ) -> Self {
        Self {
            section_count,
            widget_count,
            trace_ref_count,
            node_count,
        }
    }

    /// Number of top-level auditor navigation sections.
    #[must_use]
    pub const fn section_count(self) -> usize {
        self.section_count
    }

    /// Number of traceable widgets emitted by the projection.
    #[must_use]
    pub const fn widget_count(self) -> usize {
        self.widget_count
    }

    /// Number of source trace references preserved.
    #[must_use]
    pub const fn trace_ref_count(self) -> usize {
        self.trace_ref_count
    }

    /// Number of nodes in the emitted `elicit_ui` tree.
    #[must_use]
    pub const fn node_count(self) -> usize {
        self.node_count
    }
}

#[instrument(level = "debug", skip(evidence))]
fn ui_ir_receipt_digest(evidence: UiIrReceiptDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, UI_IR_RECEIPT_SCHEMA_ID);
    update_field(&mut hasher, evidence.contract);
    update_field(&mut hasher, &evidence.counts.section_count().to_string());
    update_field(&mut hasher, &evidence.counts.widget_count().to_string());
    update_field(&mut hasher, &evidence.counts.trace_ref_count().to_string());
    update_field(&mut hasher, &evidence.counts.node_count().to_string());
    update_field(&mut hasher, evidence.source_view_model_receipt_id);
    update_field(&mut hasher, evidence.source_view_model_receipt_digest);
    update_fields(&mut hasher, evidence.source_record_ids);
    update_fields(&mut hasher, evidence.section_ids);
    update_u64_fields(&mut hasher, evidence.section_root_widget_ids);
    update_u64_fields(&mut hasher, evidence.trace_widget_ids);
    update_fields(&mut hasher, evidence.trace_widget_ref_ids);
    update_fields(&mut hasher, evidence.trace_widget_digests);
    update_fields(&mut hasher, evidence.output_ids);
    update_counts(&mut hasher, evidence.output_kind_counts);
    update_counts(&mut hasher, evidence.role_counts);
    update_counts(&mut hasher, evidence.section_widget_counts);
    update_counts(&mut hasher, evidence.section_population_counts);
    update_counts(&mut hasher, evidence.section_trace_counts);
    update_fields(&mut hasher, evidence.viewport_ids);
    update_counts(&mut hasher, evidence.viewport_verification_counts);
    update_counts(&mut hasher, evidence.trace_source_counts);
    update_counts(&mut hasher, evidence.trace_source_kind_counts);
    update_counts(&mut hasher, evidence.source_locator_counts);
    update_counts(&mut hasher, evidence.minted_prop_counts);
    update_counts(&mut hasher, evidence.proof_token_counts);
    update_fields(&mut hasher, evidence.minted_standards);
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(hasher, fields))]
fn update_u64_fields(hasher: &mut Sha256, fields: &[u64]) {
    for field in fields {
        update_field(hasher, &field.to_string());
    }
}

#[instrument(level = "trace", skip(hasher, counts))]
fn update_counts(hasher: &mut Sha256, counts: &BTreeMap<String, usize>) {
    for (name, count) in counts {
        update_field(hasher, name);
        update_field(hasher, &count.to_string());
    }
}
