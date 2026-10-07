//! Normal run CLI output writers.

use std::{fs::File, io::Write};

use tracing::instrument;

use crate::{BelvedereError, BelvedereRunReport};

use super::{BelvedereCliOutput, write_json};
use crate::cli::{BelvedereCliOutputError, BelvedereCliUsageError};

#[instrument(level = "info", skip(output, run, writer))]
pub(super) fn write_run_output(
    output: BelvedereCliOutput,
    run: &BelvedereRunReport,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    match output {
        BelvedereCliOutput::RunCertificateJson => write_json(run.certificate(), writer),
        BelvedereCliOutput::RunCertificateFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.certificate(), file)
        }
        BelvedereCliOutput::RunManifestJson => write_json(run.manifest(), writer),
        BelvedereCliOutput::RunManifestFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.manifest(), file)
        }
        BelvedereCliOutput::RunRegistryReceiptJson => write_json(run.registry_receipt(), writer),
        BelvedereCliOutput::RunRegistryReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.registry_receipt(), file)
        }
        BelvedereCliOutput::RunAssessmentReceiptJson => {
            write_json(run.assessment_receipt(), writer)
        }
        BelvedereCliOutput::RunAssessmentReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.assessment_receipt(), file)
        }
        BelvedereCliOutput::RunLandscapeReceiptJson => write_json(run.landscape_receipt(), writer),
        BelvedereCliOutput::RunLandscapeReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.landscape_receipt(), file)
        }
        BelvedereCliOutput::RunLandscapeJson => write_json(run.landscape(), writer),
        BelvedereCliOutput::RunLandscapeFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.landscape(), file)
        }
        BelvedereCliOutput::RunViewModelJson => write_json(run.view_model(), writer),
        BelvedereCliOutput::RunViewModelFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.view_model(), file)
        }
        BelvedereCliOutput::RunViewModelReceiptJson => write_json(run.view_model_receipt(), writer),
        BelvedereCliOutput::RunViewModelReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.view_model_receipt(), file)
        }
        BelvedereCliOutput::RunUiIrJson => write_json(run.projection(), writer),
        BelvedereCliOutput::RunUiIrFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.projection(), file)
        }
        BelvedereCliOutput::RunUiIrReceiptJson => write_json(run.ui_ir_receipt(), writer),
        BelvedereCliOutput::RunUiIrReceiptFile(path) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.ui_ir_receipt(), file)
        }
        _ => Err(BelvedereCliUsageError::new().into()),
    }
}
