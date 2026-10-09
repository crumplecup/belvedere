//! Receipts for Belvedere landscape modeling exchanges.

use std::collections::BTreeMap;

use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::certify::{hex_digest, update_field, update_fields};

const LANDSCAPE_RECEIPT_SCHEMA_ID: &str = "belvedere.receipts.landscape.v1";
const LANDSCAPE_RECEIPT_ID_PREFIX: &str = "belvedere.receipt.landscape";

/// Audit receipt emitted by a landscape modeling exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct LandscapeReceipt {
    contract: String,
    #[getter(skip)]
    counts: LandscapeReceiptCounts,
    receipt_id: String,
    receipt_digest: String,
    item_kind_counts: BTreeMap<String, usize>,
    gap_kind_counts: BTreeMap<String, usize>,
    assessment_recency_counts: BTreeMap<String, usize>,
    landscape_item_ids: Vec<String>,
    source_receipt_ids: Vec<String>,
    source_receipt_digests: Vec<String>,
    source_record_ids: Vec<String>,
    warnings: Vec<String>,
}

impl LandscapeReceipt {
    /// Number of claims in the landscape.
    #[must_use]
    pub const fn claim_count(&self) -> usize {
        self.counts.claim_count()
    }

    /// Number of evidence edges in the landscape.
    #[must_use]
    pub const fn edge_count(&self) -> usize {
        self.counts.edge_count()
    }

    /// Number of explicit root claims.
    #[must_use]
    pub const fn root_count(&self) -> usize {
        self.counts.root_count()
    }

    /// Number of proof summaries in the landscape.
    #[must_use]
    pub const fn proof_count(&self) -> usize {
        self.counts.proof_count()
    }

    /// Number of assessment events in the landscape.
    #[must_use]
    pub const fn assessment_count(&self) -> usize {
        self.counts.assessment_count()
    }

    /// Number of conservative review gaps.
    #[must_use]
    pub const fn gap_count(&self) -> usize {
        self.counts.gap_count()
    }

    /// All modeled landscape counts.
    #[must_use]
    pub const fn counts(&self) -> LandscapeReceiptCounts {
        self.counts
    }
}

/// Evidence used to identify a landscape receipt.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(vis = "pub(crate)")]
pub(crate) struct LandscapeReceiptDigestEvidence<'evidence> {
    /// Exchange contract that emitted the receipt.
    contract: &'evidence str,
    /// Modeled landscape counts.
    counts: LandscapeReceiptCounts,
    /// Counts grouped by emitted landscape item kind.
    item_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by conservative gap kind.
    gap_kind_counts: &'evidence BTreeMap<String, usize>,
    /// Counts grouped by assessment recency.
    assessment_recency_counts: &'evidence BTreeMap<String, usize>,
    /// Stable landscape item IDs emitted by the exchange.
    landscape_item_ids: &'evidence [String],
    /// Upstream ingestion receipt IDs used as input evidence.
    source_receipt_ids: &'evidence [String],
    /// Upstream ingestion receipt digests used as input evidence.
    source_receipt_digests: &'evidence [String],
    /// Source record IDs represented by the landscape.
    source_record_ids: &'evidence [String],
    /// Recoverable warnings emitted by the exchange.
    warnings: &'evidence [String],
}

/// Build the stable identifier and digest for a landscape receipt.
#[instrument(level = "debug", skip(evidence))]
pub(crate) fn landscape_receipt_identity(
    evidence: LandscapeReceiptDigestEvidence<'_>,
) -> (String, String) {
    let receipt_digest = landscape_receipt_digest(evidence);
    let receipt_id = format!("{LANDSCAPE_RECEIPT_ID_PREFIX}:{receipt_digest}");
    (receipt_id, receipt_digest)
}

/// Counts emitted by a landscape modeling receipt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct LandscapeReceiptCounts {
    claim_count: usize,
    edge_count: usize,
    root_count: usize,
    proof_count: usize,
    assessment_count: usize,
    gap_count: usize,
}

impl LandscapeReceiptCounts {
    /// Build modeled landscape counts.
    #[must_use]
    pub const fn new(
        claim_count: usize,
        edge_count: usize,
        root_count: usize,
        proof_count: usize,
        assessment_count: usize,
        gap_count: usize,
    ) -> Self {
        Self {
            claim_count,
            edge_count,
            root_count,
            proof_count,
            assessment_count,
            gap_count,
        }
    }

    /// Number of claims in the landscape.
    #[must_use]
    pub const fn claim_count(self) -> usize {
        self.claim_count
    }

    /// Number of evidence edges in the landscape.
    #[must_use]
    pub const fn edge_count(self) -> usize {
        self.edge_count
    }

    /// Number of explicit root claims.
    #[must_use]
    pub const fn root_count(self) -> usize {
        self.root_count
    }

    /// Number of proof coverage entries in the landscape.
    #[must_use]
    pub const fn proof_count(self) -> usize {
        self.proof_count
    }

    /// Number of assessment events in the landscape.
    #[must_use]
    pub const fn assessment_count(self) -> usize {
        self.assessment_count
    }

    /// Number of conservative review gaps.
    #[must_use]
    pub const fn gap_count(self) -> usize {
        self.gap_count
    }
}

#[instrument(level = "debug", skip(evidence))]
fn landscape_receipt_digest(evidence: LandscapeReceiptDigestEvidence<'_>) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, LANDSCAPE_RECEIPT_SCHEMA_ID);
    update_field(&mut hasher, evidence.contract);
    update_field(&mut hasher, &evidence.counts.claim_count().to_string());
    update_field(&mut hasher, &evidence.counts.edge_count().to_string());
    update_field(&mut hasher, &evidence.counts.root_count().to_string());
    update_field(&mut hasher, &evidence.counts.proof_count().to_string());
    update_field(&mut hasher, &evidence.counts.assessment_count().to_string());
    update_field(&mut hasher, &evidence.counts.gap_count().to_string());
    update_counts(&mut hasher, evidence.item_kind_counts);
    update_counts(&mut hasher, evidence.gap_kind_counts);
    update_counts(&mut hasher, evidence.assessment_recency_counts);
    update_fields(&mut hasher, evidence.landscape_item_ids);
    update_fields(&mut hasher, evidence.source_receipt_ids);
    update_fields(&mut hasher, evidence.source_receipt_digests);
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
