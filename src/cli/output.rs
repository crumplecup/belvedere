//! CLI output contracts and writers.

use std::{fs::File, io::Write, path::PathBuf};

use serde::Serialize;
use tracing::instrument;

use crate::BelvedereError;

use super::{BelvedereCliOutputError, BelvedereCliReport, BelvedereCliUsageError};

/// Stable CLI help text.
pub const BELVEDERE_CLI_HELP: &str = "\
usage: belvedere [--help] [--certificate-json | --certificate-file <path>] <registry-dump.json> <proof-assessments.jsonl>
       belvedere diff [--certificate-json | --certificate-file <path> | --run-certificate-json | --run-certificate-file <path> | --diff-json] <previous-registry-dump.json> <previous-proof-assessments.jsonl> <current-registry-dump.json> <current-proof-assessments.jsonl>

Options:
  -h, --help                 Show this help text.
      --certificate-json     Write the requested certificate as JSON to stdout.
      --certificate-file     Write the requested certificate as JSON to a file.
      --run-certificate-json Write the full diff run certificate as JSON to stdout.
      --run-certificate-file Write the full diff run certificate as JSON to a file.
      --diff-json            Write the landscape diff as JSON to stdout.
";

/// Public CLI output contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BelvedereCliOutput {
    /// Write CLI help text.
    Help,
    /// Log the run summary through the configured tracing subscriber.
    LogSummary,
    /// Write the complete run certificate as JSON.
    RunCertificateJson,
    /// Write the complete run certificate as JSON to an artifact file.
    RunCertificateFile(PathBuf),
    /// Write the certified landscape diff as JSON.
    LandscapeDiffJson,
    /// Write the landscape diff stage certificate as JSON.
    LandscapeDiffCertificateJson,
    /// Write the landscape diff stage certificate as JSON to an artifact file.
    LandscapeDiffCertificateFile(PathBuf),
    /// Write the complete landscape diff run certificate as JSON.
    LandscapeDiffRunCertificateJson,
    /// Write the complete landscape diff run certificate as JSON to an artifact file.
    LandscapeDiffRunCertificateFile(PathBuf),
}

#[instrument(level = "info", skip(output, report, writer))]
pub(super) fn write_cli_output(
    output: BelvedereCliOutput,
    report: &BelvedereCliReport,
    writer: impl Write,
) -> Result<(), BelvedereError> {
    match (output, report) {
        (BelvedereCliOutput::Help, _) => write_cli_help(writer),
        (BelvedereCliOutput::LogSummary, _) => Ok(()),
        (BelvedereCliOutput::RunCertificateJson, BelvedereCliReport::Run(run)) => {
            write_json(run.certificate(), writer)
        }
        (BelvedereCliOutput::RunCertificateFile(path), BelvedereCliReport::Run(run)) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(run.certificate(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffCertificateJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.diff_certificate(), writer),
        (
            BelvedereCliOutput::LandscapeDiffCertificateFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.diff_certificate(), file)
        }
        (
            BelvedereCliOutput::LandscapeDiffRunCertificateJson,
            BelvedereCliReport::LandscapeDiff(diff),
        ) => write_json(diff.certificate(), writer),
        (
            BelvedereCliOutput::LandscapeDiffRunCertificateFile(path),
            BelvedereCliReport::LandscapeDiff(diff),
        ) => {
            let file = File::create(path).map_err(BelvedereCliOutputError::write)?;
            write_json(diff.certificate(), file)
        }
        (BelvedereCliOutput::LandscapeDiffJson, BelvedereCliReport::LandscapeDiff(diff)) => {
            write_json(diff.diff(), writer)
        }
        _ => Err(BelvedereCliUsageError::new().into()),
    }
}

#[instrument(level = "info", skip(writer))]
pub(super) fn write_cli_help(mut writer: impl Write) -> Result<(), BelvedereError> {
    writer
        .write_all(BELVEDERE_CLI_HELP.as_bytes())
        .map_err(BelvedereCliOutputError::write)?;
    Ok(())
}

#[instrument(level = "info", skip(value, writer))]
fn write_json(value: &impl Serialize, mut writer: impl Write) -> Result<(), BelvedereError> {
    serde_json::to_writer_pretty(&mut writer, value).map_err(BelvedereCliOutputError::serialize)?;
    writer
        .write_all(b"\n")
        .map_err(BelvedereCliOutputError::write)?;
    Ok(())
}
