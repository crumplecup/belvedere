//! Certificate metadata projected as structured UI evidence.

use amenable_core::{Certificate, ErasedEntry, Metadata};
use derive_builder::Builder;
use derive_getters::Getters;
use serde::Serialize;
use tracing::instrument;

use crate::{
    certify::BelvedereCertificationError,
    contracts::{
        ACCESSIBILITY_BASELINE_STANDARD_ID, BelvedereContract, UI_PROJECTION_FIDELITY_STANDARD_ID,
        require_transition,
    },
};

use super::UiIrProjection;

/// Stable section ID for a certificate's metadata fields.
pub const UI_IR_SECTION_CERTIFICATE_METADATA: &str = "belvedere.ui.section.certificate_metadata";

/// One metadata entry, retaining its source order and rendered value.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct MetadataUiField {
    key: String,
    value: String,
    #[getter(skip)]
    ordinal: usize,
}

impl MetadataUiField {
    /// Position in the source metadata snapshot.
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
}

/// An owned snapshot accepted by the metadata-to-UI exchange.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct MetadataUiIrInput {
    source_id: String,
    heading: String,
    fields: Vec<MetadataUiField>,
}

impl MetadataUiIrInput {
    /// Snapshot a live Amenable metadata record without losing field order.
    #[must_use]
    #[instrument(level = "debug", skip(source_id, heading, metadata))]
    pub fn from_metadata(
        source_id: impl Into<String>,
        heading: impl Into<String>,
        metadata: &impl Metadata,
    ) -> Self {
        let fields = metadata
            .snapshot()
            .iter()
            .enumerate()
            .map(|(ordinal, entry)| MetadataUiField {
                key: entry.key().to_owned(),
                value: entry.value().to_string(),
                ordinal,
            })
            .collect();
        Self {
            source_id: source_id.into(),
            heading: heading.into(),
            fields,
        }
    }

    /// Snapshot an issued certificate's frozen metadata entries.
    #[must_use]
    #[instrument(level = "debug", skip(source_id, certificate))]
    pub fn from_certificate<C: Certificate>(source_id: impl Into<String>, certificate: &C) -> Self {
        let fields = certificate
            .entries()
            .enumerate()
            .map(|(ordinal, entry)| MetadataUiField {
                key: entry.key().to_owned(),
                value: entry.value().to_owned(),
                ordinal,
            })
            .collect();
        Self {
            source_id: source_id.into(),
            heading: format!(
                "Certificate {} for {}",
                certificate.id(),
                certificate.subject()
            ),
            fields,
        }
    }
}

/// Audit receipt for a metadata-to-UI projection.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Builder, Serialize)]
#[builder(vis = "pub(crate)", setter(into))]
pub struct MetadataUiIrReceipt {
    source_id: String,
    input_digest: String,
    output_digest: String,
    #[getter(skip)]
    field_count: usize,
    #[builder(default)]
    transition_checks: Vec<crate::certify::BelvedereInvariantCheck>,
}

impl crate::certify::BelvedereTransitionReceipt for MetadataUiIrReceipt {
    #[instrument(level = "trace", skip(self))]
    fn transition_checks(&self) -> &[crate::certify::BelvedereInvariantCheck] {
        &self.transition_checks
    }

    #[instrument(level = "trace", skip(self, checks))]
    fn record_transition_checks(
        &mut self,
        checks: Vec<crate::certify::BelvedereInvariantCheck>,
    ) -> Result<(), BelvedereCertificationError> {
        crate::certify::require_transition_checks(&checks)?;
        self.transition_checks = checks;
        Ok(())
    }
}

impl MetadataUiIrReceipt {
    /// Number of metadata entries represented in the UI tree.
    #[must_use]
    pub const fn field_count(&self) -> usize {
        self.field_count
    }
}

/// Contract for rendering Amenable metadata and certificate fields in UI IR.
pub struct ElicitUiMetadataProjectionContract;

impl BelvedereContract for ElicitUiMetadataProjectionContract {
    type Input = MetadataUiIrInput;
    type Output = UiIrProjection;
    type Receipt = MetadataUiIrReceipt;

    const NAME: &'static str = "belvedere.ui_ir.metadata_projection";
    const ASSERTION_NAMES: &'static [&'static str] = &[
        "metadata_fields_preserved_in_order",
        "metadata_receipt_source_binding",
        "metadata_receipt_count_binding",
    ];

    #[instrument(level = "trace", skip(input, output, receipt))]
    fn verify_transition(
        input: &Self::Input,
        output: &Self::Output,
        receipt: &Self::Receipt,
    ) -> Result<Vec<crate::certify::BelvedereInvariantCheck>, BelvedereCertificationError> {
        let widgets = output.trace_widgets();
        let fields_preserved = widgets.len() == input.fields().len() + 1
            && widgets.first().is_some_and(|widget| {
                widget.label() == input.heading()
                    && widget.source_record_ids() == &vec![input.source_id().to_owned()]
            })
            && input
                .fields()
                .iter()
                .zip(widgets.iter().skip(1))
                .all(|(field, widget)| {
                    widget.label() == &format!("{}: {}", field.key(), field.value())
                        && widget.item_id() == &format!("{}:{}", field.ordinal(), field.key())
                        && widget.source_record_ids() == &vec![input.source_id().to_owned()]
                });
        require_transition::<Self>([
            ("metadata_fields_preserved_in_order", fields_preserved),
            (
                "metadata_receipt_source_binding",
                receipt.source_id() == input.source_id(),
            ),
            (
                "metadata_receipt_count_binding",
                receipt.field_count() == input.fields().len(),
            ),
        ])
    }

    #[instrument(level = "trace", skip(receipt))]
    fn verify_receipt(receipt: &Self::Receipt) -> Result<(), BelvedereCertificationError> {
        crate::certify::require_transition_checks(receipt.transition_checks())
    }

    #[instrument(level = "trace")]
    fn requires() -> &'static [&'static str] {
        &[
            "input is an ordered snapshot of Amenable Metadata or Certificate entries",
            "caller supplies a source identity for traceability",
        ]
    }

    #[instrument(level = "trace")]
    fn ensures() -> &'static [&'static str] {
        &[
            "every source entry appears exactly once, in source order, with its key and rendered value preserved",
            "each metadata field has a traceable UI widget linked to the source identity",
            "the emitted tree is an elicit_ui VerifiedTree with WCAG proof",
            "the receipt binds the input snapshot and projected field widgets by digest",
        ]
    }

    #[instrument(level = "trace")]
    fn standard_ids() -> &'static [&'static str] {
        &[
            UI_PROJECTION_FIDELITY_STANDARD_ID,
            ACCESSIBILITY_BASELINE_STANDARD_ID,
        ]
    }
}
