//! Claim and proof evidence carried by the landscape graph.

use std::collections::BTreeSet;

use derive_getters::Getters;
use derive_new::new;
use tracing::instrument;

use crate::contracts::{
    AssessmentRecord, Recommendation, ResolutionPath, Rubric, WitnessExportRecord,
};

/// A claim, proof harness, root, or evidence node in the defensive landscape.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct LandscapeClaim {
    id: String,
    #[getter(skip)]
    bases: BTreeSet<String>,
    #[getter(skip)]
    verifiers: BTreeSet<String>,
    contracts: Vec<LandscapeContractFragment>,
    witness_surfaces: Vec<WitnessSurfaceSummary>,
    kani_proofs: Vec<KaniProofSummary>,
    assessment_ids: Vec<String>,
}

impl LandscapeClaim {
    /// Create an empty landscape claim.
    #[must_use]
    #[instrument(level = "debug", skip(id))]
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            bases: BTreeSet::new(),
            verifiers: BTreeSet::new(),
            contracts: Vec::new(),
            witness_surfaces: Vec::new(),
            kani_proofs: Vec::new(),
            assessment_ids: Vec::new(),
        }
    }

    /// Basis claim IDs.
    #[must_use]
    pub const fn bases(&self) -> &BTreeSet<String> {
        &self.bases
    }

    /// Verifiers with proof coverage for this claim.
    #[must_use]
    pub const fn proofs(&self) -> &BTreeSet<String> {
        &self.verifiers
    }

    /// Whether this claim is an explicit root of the modeled graph.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn is_root(&self) -> bool {
        self.bases.is_empty() || self.bases.contains(&self.id)
    }

    #[instrument(level = "debug", skip(self, basis))]
    pub(crate) fn add_basis(&mut self, basis: impl Into<String>) {
        self.bases.insert(basis.into());
    }

    #[instrument(level = "debug", skip(self, verifier))]
    pub(crate) fn add_verifier(&mut self, verifier: impl Into<String>) {
        self.verifiers.insert(verifier.into());
    }

    #[instrument(level = "debug", skip(self, contract))]
    pub(crate) fn add_contract(&mut self, contract: LandscapeContractFragment) {
        self.contracts.push(contract);
    }

    #[instrument(level = "debug", skip(self, witness_surface))]
    pub(crate) fn add_witness_surface(&mut self, witness_surface: WitnessSurfaceSummary) {
        self.witness_surfaces.push(witness_surface);
    }

    #[instrument(level = "debug", skip(self, proof))]
    pub(crate) fn add_kani_proof(&mut self, proof: KaniProofSummary) {
        self.kani_proofs.push(proof);
    }

    #[instrument(level = "debug", skip(self, assessment_id))]
    pub(crate) fn add_assessment(&mut self, assessment_id: impl Into<String>) {
        self.assessment_ids.push(assessment_id.into());
    }
}

/// A directed evidence edge from a claim to one of its bases.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct ClaimEdge {
    #[new(into)]
    claim_id: String,
    #[new(into)]
    basis_id: String,
    #[getter(skip)]
    source_index: usize,
}

impl ClaimEdge {
    /// Source index from Amenable's evidence-link record.
    #[must_use]
    pub const fn source_index(&self) -> usize {
        self.source_index
    }
}

/// Backend-native contract fragment attached to a claim.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct LandscapeContractFragment {
    #[new(into)]
    verifier: String,
    #[new(into)]
    kind: String,
    #[new(into)]
    fragment: String,
}

/// Witness export support summary attached to a claim.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct WitnessSurfaceSummary {
    verifier: String,
    destination_module: String,
    support_kind: String,
    #[getter(skip)]
    trivial: usize,
    #[getter(skip)]
    checked: usize,
    #[getter(skip)]
    trusted: usize,
    #[getter(skip)]
    opaque: usize,
}

impl WitnessSurfaceSummary {
    /// Create a witness surface summary from a decoded registry record.
    #[must_use]
    #[instrument(level = "debug", skip(record))]
    pub fn from_record(record: &WitnessExportRecord) -> Self {
        Self {
            verifier: record.verifier().to_owned(),
            destination_module: record.destination_module().to_owned(),
            support_kind: record.support_kind().to_owned(),
            trivial: record.trivial(),
            checked: record.checked(),
            trusted: record.trusted(),
            opaque: record.opaque(),
        }
    }

    /// Trivial support count.
    #[must_use]
    pub const fn trivial(&self) -> usize {
        self.trivial
    }

    /// Checked support count.
    #[must_use]
    pub const fn checked(&self) -> usize {
        self.checked
    }

    /// Trusted support count.
    #[must_use]
    pub const fn trusted(&self) -> usize {
        self.trusted
    }

    /// Opaque support count.
    #[must_use]
    pub const fn opaque(&self) -> usize {
        self.opaque
    }
}

/// Kani proof registration attached to a claim.
#[derive(Debug, Clone, PartialEq, Eq, Getters, new)]
pub struct KaniProofSummary {
    #[new(into)]
    id: String,
    #[new(into)]
    harness: String,
    #[new(into)]
    package: String,
}

/// Normalized assessment event attached to a claim or proof.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct LandscapeAssessment {
    id: String,
    proof_id: String,
    reviewer: String,
    #[getter(skip)]
    timestamp: u64,
    #[getter(skip)]
    rubric: Rubric,
    #[getter(skip)]
    recommendation: Recommendation,
    #[getter(skip)]
    resolution_path: Option<ResolutionPath>,
    comment: String,
    #[getter(skip)]
    source_line: usize,
}

impl LandscapeAssessment {
    /// Create a normalized assessment event from a decoded assessment record.
    #[must_use]
    #[instrument(level = "debug", skip(record))]
    pub fn from_record(record: &AssessmentRecord) -> Self {
        let id = record.assessment_id().map_or_else(
            || format!("assessment:{}@{}", record.proof_id(), record.timestamp()),
            ToOwned::to_owned,
        );
        Self {
            id,
            proof_id: record.proof_id().to_owned(),
            reviewer: record.reviewer().to_owned(),
            timestamp: record.timestamp(),
            rubric: record.rubric(),
            recommendation: record.recommendation(),
            resolution_path: record.resolution_path(),
            comment: record.comment().to_owned(),
            source_line: record.source_line(),
        }
    }

    /// Normalized timestamp.
    #[must_use]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Rubric scores.
    #[must_use]
    pub const fn rubric(&self) -> Rubric {
        self.rubric
    }

    /// Reviewer recommendation.
    #[must_use]
    pub const fn recommendation(&self) -> Recommendation {
        self.recommendation
    }

    /// Recommended resolution path.
    #[must_use]
    pub const fn resolution_path(&self) -> Option<ResolutionPath> {
        self.resolution_path
    }

    /// Source line from the JSON Lines artifact.
    #[must_use]
    pub const fn source_line(&self) -> usize {
        self.source_line
    }
}
