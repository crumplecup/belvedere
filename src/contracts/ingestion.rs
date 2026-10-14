//! Ingestion contracts and evidence types.

use std::{
    fmt::{self, Display, Formatter},
    path::PathBuf,
};

use derive_getters::Getters;
use serde::{Serialize, Serializer};
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::{
    certify::IngestionReceipt,
    contracts::{BelvedereContract, INPUT_FIDELITY_STANDARD_ID, SCHEMA_HONESTY_STANDARD_ID},
};

mod assessment;
mod registry;
mod unsupported;

pub use assessment::{
    AssessmentRecord, DecodedAssessmentStream, Recommendation, ResolutionPath, Rubric,
};
pub use registry::{
    ContractRecordRecord, DecodedRegistrySnapshot, EvidenceLinkRecord, KaniProofRecord,
    ProofExecutionStatus, ProofRecordRecord, WitnessExportRecord,
};
pub use unsupported::UnsupportedField;

/// Supported Amenable artifact families at the ingestion boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArtifactKind {
    /// JSON emitted by `amenable dump-registry`.
    RegistryDump,
    /// JSON Lines proof assessment stream.
    ProofAssessments,
}

impl ArtifactKind {
    /// Stable artifact-kind identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RegistryDump => "registry_dump",
            Self::ProofAssessments => "proof_assessments",
        }
    }
}

impl Display for ArtifactKind {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Serialize for ArtifactKind {
    #[instrument(level = "trace", skip(self, serializer))]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// Accepted schema roots for Amenable artifacts Belvedere can ingest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AcceptedAmenableArtifactFormat {
    /// Current `dump-registry` JSON object shape.
    RegistryDumpJson,
    /// Current and legacy proof assessment JSON Lines shape.
    ProofAssessmentJsonLines,
}

impl AcceptedAmenableArtifactFormat {
    /// Stable schema identifier for receipts and future Amenable registration.
    pub const fn schema_id(self) -> &'static str {
        match self {
            Self::RegistryDumpJson => "amenable.registry_dump_json",
            Self::ProofAssessmentJsonLines => "amenable.proof_assessment_jsonl",
        }
    }

    /// The artifact kind this format accepts.
    pub const fn artifact_kind(self) -> ArtifactKind {
        match self {
            Self::RegistryDumpJson => ArtifactKind::RegistryDump,
            Self::ProofAssessmentJsonLines => ArtifactKind::ProofAssessments,
        }
    }
}

impl Display for AcceptedAmenableArtifactFormat {
    #[instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.schema_id())
    }
}

impl Serialize for AcceptedAmenableArtifactFormat {
    #[instrument(level = "trace", skip(self, serializer))]
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.schema_id())
    }
}

/// Raw input evidence for an Amenable artifact.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct RawAmenableArtifact {
    path: PathBuf,
    #[getter(skip)]
    kind: ArtifactKind,
    bytes: Vec<u8>,
    digest: String,
}

impl RawAmenableArtifact {
    /// Create raw artifact evidence from bytes already read by the caller.
    #[must_use]
    #[instrument(level = "debug", skip(path, kind))]
    pub fn new(path: impl Into<PathBuf>, kind: ArtifactKind, bytes: Vec<u8>) -> Self {
        let digest = sha256_hex(&bytes);
        Self {
            path: path.into(),
            kind,
            bytes,
            digest,
        }
    }

    /// Declared artifact kind.
    #[must_use]
    pub const fn kind(&self) -> ArtifactKind {
        self.kind
    }
}

/// Ingestion contract for registry dump JSON.
pub struct RegistryDumpIngestionContract;

impl BelvedereContract for RegistryDumpIngestionContract {
    type Input = RawAmenableArtifact;
    type Output = DecodedRegistrySnapshot;
    type Receipt = IngestionReceipt;

    const NAME: &'static str = "belvedere.ingest.registry_dump";

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "input artifact kind is registry_dump",
            "input bytes are valid JSON",
            "artifact is governed by the accepted registry dump schema root",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "decoded evidence links preserve name, basis, and index",
            "decoded proof records preserve evidence and verifier identity",
            "decoded contract records preserve evidence, verifier, kind, and fragment",
            "decoded witness and Kani registrations retain stable source identities including their distinct origins",
            "unsupported fields are preserved in the ingestion receipt",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[INPUT_FIDELITY_STANDARD_ID, SCHEMA_HONESTY_STANDARD_ID]
    }
}

/// Ingestion contract for proof assessment JSON Lines.
pub struct AssessmentStreamIngestionContract;

impl BelvedereContract for AssessmentStreamIngestionContract {
    type Input = RawAmenableArtifact;
    type Output = DecodedAssessmentStream;
    type Receipt = IngestionReceipt;

    const NAME: &'static str = "belvedere.ingest.proof_assessments";

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "input artifact kind is proof_assessments",
            "each non-empty input line is valid JSON",
            "artifact is governed by the accepted assessment JSON Lines schema root",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "decoded assessments preserve proof ID, reviewer, timestamp, recommendation, and comment",
            "current assessment records preserve assessment ID and resolution path",
            "legacy assessment records preserve schema version and timestamp field spelling",
            "unsupported fields are preserved in the ingestion receipt",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[INPUT_FIDELITY_STANDARD_ID, SCHEMA_HONESTY_STANDARD_ID]
    }
}

#[instrument(level = "debug")]
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}
