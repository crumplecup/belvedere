//! Source record navigation projections.

use derive_new::new;
use tracing::instrument;

use crate::{
    contracts::{ArtifactKind, SourceNavigationItem, SourceNavigationItemBuilder, TraceRef},
    error::BelvedereConstructionError,
};

#[instrument(level = "debug", skip(source_record_ids))]
pub(super) fn project_source_navigation(
    source_record_ids: &[String],
) -> Result<Vec<SourceNavigationItem>, BelvedereConstructionError> {
    source_record_ids
        .iter()
        .map(|source_record_id| {
            let source = source_record(source_record_id);
            SourceNavigationItemBuilder::default()
                .source_record_id(source_record_id)
                .artifact_kind(source.artifact_kind)
                .source_kind(source.kind)
                .target(source.target)
                .source_locator(source.locator)
                .label(source.label)
                .trace(TraceRef::new(
                    format!("belvedere.source_record:{source_record_id}"),
                    vec![source_record_id.to_owned()],
                ))
                .build()
                .map_err(|error| BelvedereConstructionError::new("SourceNavigationItem", error))
        })
        .collect()
}

#[instrument(level = "trace", skip(source_record_id))]
fn source_record(source_record_id: &str) -> SourceRecordNavigation {
    if let Some(rest) = source_record_id.strip_prefix("evidence_link:") {
        let locator = rest.split_once('#').map_or_else(
            || format!("registry_dump.evidence_links[id={rest}]"),
            |(name, index)| format!("registry_dump.evidence_links[name={name}, index={index}]"),
        );
        SourceRecordNavigation::new(
            ArtifactKind::RegistryDump,
            "evidence_link",
            rest,
            locator,
            format!("Evidence link {rest}"),
        )
    } else if let Some(rest) = source_record_id.strip_prefix("proof_record:") {
        let locator = rest.split_once('@').map_or_else(
            || format!("registry_dump.proof_records[id={rest}]"),
            |(evidence, verifier)| {
                format!("registry_dump.proof_records[evidence={evidence}, verifier={verifier}]")
            },
        );
        SourceRecordNavigation::new(
            ArtifactKind::RegistryDump,
            "proof_record",
            rest,
            locator,
            format!("Proof record {rest}"),
        )
    } else if let Some(rest) = source_record_id.strip_prefix("contract_record:") {
        let (locator, label) = rest
            .split_once('@')
            .and_then(|(evidence, registration)| {
                registration.split_once(':').and_then(|(verifier, kind_and_digest)| {
                    kind_and_digest.split_once('#').map(|(kind, digest)| {
                        (
                            format!(
                                "registry_dump.contract_records[evidence={evidence}, verifier={verifier}, kind={kind}, fragment_sha256={digest}]"
                            ),
                            format!(
                                "Contract record {evidence}@{verifier}:{kind} ({})",
                                digest.get(..12).unwrap_or(digest)
                            ),
                        )
                    })
                })
            })
            .unwrap_or_else(|| {
                (
                    format!("registry_dump.contract_records[id={rest}]"),
                    format!("Contract record {rest}"),
                )
            });
        SourceRecordNavigation::new(
            ArtifactKind::RegistryDump,
            "contract_record",
            rest,
            locator,
            label,
        )
    } else if let Some(rest) = source_record_id.strip_prefix("witness_export:") {
        let locator = rest
            .split_once('@')
            .and_then(|(evidence, rest)| {
                rest.split_once(':').map(|(verifier, destination_module)| {
                    format!(
                        "registry_dump.witness_export_records[evidence={evidence}, verifier={verifier}, destination_module={destination_module}]"
                    )
                })
            })
            .unwrap_or_else(|| format!("registry_dump.witness_export_records[id={rest}]"));
        SourceRecordNavigation::new(
            ArtifactKind::RegistryDump,
            "witness_export",
            rest,
            locator,
            format!("Witness export {rest}"),
        )
    } else if let Some(rest) = source_record_id.strip_prefix("kani_proof:") {
        let (target, locator) = rest
            .split_once('@')
            .and_then(|(proof_id, registration)| {
                registration.split_once(':').map(|(package, harness)| {
                    (
                        proof_id,
                        format!(
                            "registry_dump.kani_proofs[id={proof_id}, package={package}, harness={harness}]"
                        ),
                    )
                })
            })
            .unwrap_or_else(|| (rest, format!("registry_dump.kani_proofs[id={rest}]")));
        SourceRecordNavigation::new(
            ArtifactKind::RegistryDump,
            "kani_proof",
            target,
            locator,
            format!("Kani proof {rest}"),
        )
    } else if let Some(rest) = source_record_id.strip_prefix("assessment:") {
        let locator = rest.split_once('@').map_or_else(
            || format!("assessment_jsonl.assessment_id={rest}"),
            |(proof_id, timestamp)| {
                format!("assessment_jsonl.proof_id={proof_id}, timestamp={timestamp}")
            },
        );
        SourceRecordNavigation::new(
            ArtifactKind::ProofAssessments,
            "assessment",
            rest,
            locator,
            format!("Assessment {rest}"),
        )
    } else {
        SourceRecordNavigation::new(
            ArtifactKind::RegistryDump,
            "source_record",
            source_record_id,
            format!("registry_dump.source_records[id={source_record_id}]"),
            format!("Source record {source_record_id}"),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, new)]
struct SourceRecordNavigation {
    artifact_kind: ArtifactKind,
    #[new(into)]
    kind: String,
    #[new(into)]
    target: String,
    #[new(into)]
    locator: String,
    #[new(into)]
    label: String,
}
