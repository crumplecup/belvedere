//! Stage certificates for ingestion exchanges.

use tracing::instrument;

use crate::{
    certify::{BelvederePipelineStage, IngestionReceipt, ingestion_record_invariant_checks},
    contracts::{
        AssessmentStreamIngestionContract, BelvedereContract, RegistryDumpIngestionContract,
    },
};

use super::{BelvedereStageCertificate, StageCertificateEvidence};
use crate::certify::contract_metadata::{
    contract_guarantees, contract_requirements, contract_standards,
};

impl BelvedereStageCertificate {
    /// Certificate the registry dump ingestion stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn registry_ingestion(receipt: &IngestionReceipt) -> Self {
        Self::ingestion::<RegistryDumpIngestionContract>(
            BelvederePipelineStage::RegistryIngestion,
            receipt,
        )
    }

    /// Certificate the assessment stream ingestion stage.
    #[instrument(level = "debug", skip(receipt))]
    #[must_use]
    pub fn assessment_ingestion(receipt: &IngestionReceipt) -> Self {
        Self::ingestion::<AssessmentStreamIngestionContract>(
            BelvederePipelineStage::AssessmentIngestion,
            receipt,
        )
    }

    #[instrument(level = "debug", skip(stage, receipt))]
    fn ingestion<C: BelvedereContract>(
        stage: BelvederePipelineStage,
        receipt: &IngestionReceipt,
    ) -> Self {
        let mut counts = receipt.decoded_counts().clone();
        counts.extend(receipt.source_record_kind_counts().clone());
        counts.extend(receipt.unsupported_field_record_counts().clone());
        counts.insert(
            "unsupported_fields".to_owned(),
            receipt.unsupported_fields().len(),
        );
        counts.insert("warnings".to_owned(), receipt.warnings().len());
        let additional_invariant_checks = ingestion_record_invariant_checks(
            receipt.decoded_counts(),
            receipt.source_record_kind_counts(),
            receipt.source_record_ids(),
        );
        Self::new(StageCertificateEvidence {
            stage,
            contract: receipt.contract().to_owned(),
            standards: contract_standards::<C>(),
            requires: contract_requirements::<C>(),
            ensures: contract_guarantees::<C>(),
            input_ids: vec![format!(
                "{}:{}:{}:{}",
                receipt.artifact_kind(),
                receipt.format(),
                receipt.path().display(),
                receipt.digest()
            )],
            input_digests: vec![receipt.digest().to_owned()],
            output_ids: receipt.source_record_ids().clone(),
            counts,
            additional_invariant_checks,
            warning_count: receipt.warnings().len(),
        })
    }
}
