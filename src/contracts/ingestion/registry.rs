//! Registry dump evidence decoded at the ingestion boundary.

use std::collections::BTreeMap;

use derive_getters::Getters;
use serde::Deserialize;
use tracing::instrument;

use super::unsupported::{UnsupportedField, unsupported};

/// Decoded `dump-registry` evidence.
#[derive(Debug, Clone, PartialEq, Deserialize, Getters)]
pub struct DecodedRegistrySnapshot {
    #[serde(default)]
    evidence_links: Vec<EvidenceLinkRecord>,
    #[serde(default)]
    proof_records: Vec<ProofRecordRecord>,
    #[serde(default)]
    contract_records: Vec<ContractRecordRecord>,
    #[serde(default)]
    witness_export_records: Vec<WitnessExportRecord>,
    #[serde(default)]
    kani_proofs: Vec<KaniProofRecord>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl DecodedRegistrySnapshot {
    /// Stable source record identifiers derived from decoded content.
    #[instrument(level = "debug", skip(self))]
    #[must_use]
    pub fn source_record_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        ids.extend(
            self.evidence_links
                .iter()
                .map(|record| format!("evidence_link:{}#{}", record.name, record.index)),
        );
        ids.extend(
            self.proof_records
                .iter()
                .map(|record| format!("proof_record:{}@{}", record.evidence, record.verifier)),
        );
        ids.extend(self.contract_records.iter().map(|record| {
            format!(
                "contract_record:{}@{}:{}",
                record.evidence, record.verifier, record.kind
            )
        }));
        ids.extend(self.witness_export_records.iter().map(|record| {
            format!(
                "witness_export:{}@{}:{}",
                record.evidence, record.verifier, record.destination_module
            )
        }));
        ids.extend(
            self.kani_proofs
                .iter()
                .map(|record| format!("kani_proof:{}", record.id)),
        );
        ids
    }

    /// Count decoded record groups for receipts.
    #[instrument(level = "trace", skip(self))]
    #[must_use]
    pub fn decoded_counts(&self) -> BTreeMap<String, usize> {
        BTreeMap::from([
            ("evidence_links".to_owned(), self.evidence_links.len()),
            ("proof_records".to_owned(), self.proof_records.len()),
            ("contract_records".to_owned(), self.contract_records.len()),
            (
                "witness_export_records".to_owned(),
                self.witness_export_records.len(),
            ),
            ("kani_proofs".to_owned(), self.kani_proofs.len()),
        ])
    }

    /// Uninterpreted fields preserved during registry ingestion.
    #[instrument(level = "debug", skip(self))]
    #[must_use]
    pub fn unsupported_fields(&self) -> Vec<UnsupportedField> {
        let mut fields = unsupported("registry", &self.extra);
        fields.extend(
            self.evidence_links
                .iter()
                .enumerate()
                .flat_map(|(index, record)| {
                    unsupported(format!("evidence_links[{index}]"), &record.extra)
                }),
        );
        fields.extend(
            self.proof_records
                .iter()
                .enumerate()
                .flat_map(|(index, record)| {
                    unsupported(format!("proof_records[{index}]"), &record.extra)
                }),
        );
        fields.extend(
            self.contract_records
                .iter()
                .enumerate()
                .flat_map(|(index, record)| {
                    unsupported(format!("contract_records[{index}]"), &record.extra)
                }),
        );
        fields.extend(self.witness_export_records.iter().enumerate().flat_map(
            |(index, record)| {
                unsupported(format!("witness_export_records[{index}]"), &record.extra)
            },
        ));
        fields.extend(
            self.kani_proofs
                .iter()
                .enumerate()
                .flat_map(|(index, record)| {
                    unsupported(format!("kani_proofs[{index}]"), &record.extra)
                }),
        );
        fields
    }
}

/// One evidence link from the registry dump.
#[derive(Debug, Clone, PartialEq, Deserialize, Getters)]
pub struct EvidenceLinkRecord {
    name: String,
    basis: String,
    #[getter(skip)]
    index: usize,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl EvidenceLinkRecord {
    /// Basis index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }
}

/// One proof registration from the registry dump.
#[derive(Debug, Clone, PartialEq, Deserialize, Getters)]
pub struct ProofRecordRecord {
    evidence: String,
    verifier: String,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl ProofRecordRecord {}

/// One contract fragment from the registry dump.
#[derive(Debug, Clone, PartialEq, Deserialize, Getters)]
pub struct ContractRecordRecord {
    evidence: String,
    verifier: String,
    kind: String,
    fragment: String,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl ContractRecordRecord {}

/// One witness export record from the registry dump.
#[derive(Debug, Clone, PartialEq, Deserialize, Getters)]
pub struct WitnessExportRecord {
    verifier: String,
    evidence: String,
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
    artifact: serde_json::Value,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl WitnessExportRecord {
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

/// One Kani proof registration from the registry dump.
#[derive(Debug, Clone, PartialEq, Deserialize, Getters)]
pub struct KaniProofRecord {
    id: String,
    harness: String,
    package: String,
    #[serde(default = "default_proof_execution_status")]
    #[getter(skip)]
    status: ProofExecutionStatus,
    #[serde(default)]
    #[getter(skip)]
    scope: Option<String>,
    #[serde(default)]
    #[getter(skip)]
    failure_reason: Option<String>,
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl KaniProofRecord {
    /// Execution status reported for this proof.
    #[must_use]
    pub const fn status(&self) -> ProofExecutionStatus {
        self.status
    }

    /// Scope label reported for this proof, if present.
    #[must_use]
    #[instrument(level = "trace", skip(self))]
    pub fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }

    /// Failure or timeout reason reported for this proof, if present.
    #[must_use]
    #[instrument(level = "trace", skip(self))]
    pub fn failure_reason(&self) -> Option<&str> {
        self.failure_reason.as_deref()
    }
}

/// Proof execution status preserved from backend records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofExecutionStatus {
    /// Backend reported the proof as verified.
    Verified,
    /// Backend reported a failing proof.
    Failed,
    /// Backend did not complete within its configured limit.
    TimedOut,
    /// Backend skipped the proof intentionally.
    Skipped,
    /// Backend status is unknown to Belvedere.
    Unknown,
}

impl ProofExecutionStatus {
    /// Human-readable status label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::Failed => "failed",
            Self::TimedOut => "timed out",
            Self::Skipped => "skipped",
            Self::Unknown => "unknown",
        }
    }

    /// Whether this status provides modeled proof coverage.
    #[must_use]
    pub const fn provides_coverage(self) -> bool {
        matches!(self, Self::Verified)
    }
}

const fn default_proof_execution_status() -> ProofExecutionStatus {
    ProofExecutionStatus::Verified
}
