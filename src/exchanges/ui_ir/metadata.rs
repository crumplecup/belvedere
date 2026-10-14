//! Direct Amenable metadata and certificate projection into `elicit_ui`.

use elicit_ui::{AccessKitUiBackend, Viewport};
use elicitation::Established;
use sha2::{Digest as _, Sha256};
use tracing::instrument;

use crate::{
    certify::{hex_digest, update_field},
    contracts::{
        ElicitUiMetadataProjectionContract, MetadataUiIrInput, MetadataUiIrReceipt,
        MetadataUiIrReceiptBuilder, TraceRef, UI_IR_SECTION_CERTIFICATE_METADATA, UiIrProjection,
        UiIrProjectionBuilder, UiIrProofBundle,
    },
    exchanges::BelvedereExchange,
};

use super::{
    UI_IR_PRIMARY_VIEWPORT, UiIrError, minted_standards, trace_widgets::labeled_widget,
    verify_viewports, widgets::section,
};

const INPUT_SCHEMA: &str = "belvedere.metadata_ui_ir.input.v1";
const OUTPUT_SCHEMA: &str = "belvedere.metadata_ui_ir.output.v1";

/// Exchange from an Amenable metadata snapshot into verified UI IR.
#[derive(Debug, Clone, Copy, Default)]
pub struct ElicitUiMetadataProjection;

impl ElicitUiMetadataProjection {
    /// Project metadata or certificate fields into a verified UI tree.
    ///
    /// # Errors
    ///
    /// Returns an error if `elicit_ui` rejects a field label or layout.
    #[instrument(level = "debug", skip(self, input))]
    pub fn project(
        &self,
        input: MetadataUiIrInput,
    ) -> Result<(UiIrProjection, MetadataUiIrReceipt), UiIrError> {
        <Self as BelvedereExchange<ElicitUiMetadataProjectionContract>>::exchange(self, input)
    }
}

impl BelvedereExchange<ElicitUiMetadataProjectionContract> for ElicitUiMetadataProjection {
    type Error = UiIrError;

    #[instrument(level = "trace", skip(self, input))]
    fn transform(
        &self,
        input: &MetadataUiIrInput,
    ) -> Result<(UiIrProjection, MetadataUiIrReceipt), Self::Error> {
        let input_digest = digest_input(input);
        let backend = AccessKitUiBackend::new();
        let mut trace_widgets = Vec::new();
        let mut widgets = Vec::with_capacity(input.fields().len() + 1);
        let trace = TraceRef::new(input.source_id(), vec![input.source_id().to_owned()]);
        widgets.push(labeled_widget(
            &backend,
            UI_IR_SECTION_CERTIFICATE_METADATA,
            "region",
            input.heading().to_owned(),
            input.source_id(),
            trace.clone(),
            &mut trace_widgets,
        )?);
        for field in input.fields() {
            widgets.push(labeled_widget(
                &backend,
                UI_IR_SECTION_CERTIFICATE_METADATA,
                "status",
                format!("{}: {}", field.key(), field.value()),
                format!("{}:{}", field.ordinal(), field.key()),
                trace.clone(),
                &mut trace_widgets,
            )?);
        }
        let sections = vec![section(
            &backend,
            UI_IR_SECTION_CERTIFICATE_METADATA,
            "certificate metadata",
            widgets,
        )?];
        let tree = backend.to_verified_tree(Viewport::new(
            UI_IR_PRIMARY_VIEWPORT.0,
            UI_IR_PRIMARY_VIEWPORT.1,
        ));
        let proofs = UiIrProofBundle::new(
            Established::<elicit_ui::IrSourced>::assert(),
            Established::<elicit_ui::WcagVerified>::prove(&tree),
        );
        let viewport_verifications = verify_viewports(&backend);
        let output_digest = digest_output(&trace_widgets);
        let receipt = MetadataUiIrReceiptBuilder::default()
            .source_id(input.source_id())
            .input_digest(input_digest)
            .output_digest(output_digest)
            .field_count(input.fields().len())
            .build()
            .map_err(|error| UiIrError::construction("MetadataUiIrReceipt", error))?;
        let projection = UiIrProjectionBuilder::default()
            .tree(tree)
            .sections(sections)
            .trace_widgets(trace_widgets)
            .viewport_verifications(viewport_verifications)
            .minted_standards(minted_standards())
            .proofs(proofs)
            .build()
            .map_err(|error| UiIrError::construction("UiIrProjection", error))?;
        Ok((projection, receipt))
    }
}

#[instrument(level = "trace", skip(input))]
fn digest_input(input: &MetadataUiIrInput) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, INPUT_SCHEMA);
    update_field(&mut hasher, input.source_id());
    update_field(&mut hasher, input.heading());
    update_field(&mut hasher, &input.fields().len().to_string());
    for field in input.fields() {
        update_field(&mut hasher, &field.ordinal().to_string());
        update_field(&mut hasher, field.key());
        update_field(&mut hasher, field.value());
    }
    hex_digest(hasher)
}

#[instrument(level = "trace", skip(trace_widgets))]
fn digest_output(trace_widgets: &[crate::contracts::TraceWidgetRef]) -> String {
    let mut hasher = Sha256::new();
    update_field(&mut hasher, OUTPUT_SCHEMA);
    update_field(&mut hasher, &trace_widgets.len().to_string());
    for widget in trace_widgets {
        update_field(&mut hasher, widget.trace_widget_digest());
    }
    hex_digest(hasher)
}
