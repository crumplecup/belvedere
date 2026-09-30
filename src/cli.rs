//! Command-line entrypoints for Belvedere.

use std::{
    io::{self, Write},
    panic::Location,
    process::ExitCode,
};

use derive_getters::Getters;
use thiserror::Error;
use tracing::{error, info, instrument};

use crate::{
    BelvedereError, BelvedereRunCertificate, BelvedereRunInput, BelvedereRunInvocation,
    BelvedereRunReport, run_belvedere_pipeline,
};

/// Parsed Belvedere command-line invocation.
#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct BelvedereCli {
    input: BelvedereRunInput,
    #[getter(skip)]
    output: BelvedereCliOutput,
}

impl BelvedereCli {
    /// Parse a Belvedere pipeline run from positional artifact paths.
    ///
    /// # Errors
    ///
    /// Returns a usage error unless exactly two artifact paths are supplied.
    #[instrument(level = "debug", skip(args))]
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self, BelvedereError> {
        let arguments = args.into_iter().collect::<Vec<_>>();
        let mut output = BelvedereCliOutput::LogSummary;
        let mut paths = Vec::new();

        for arg in &arguments {
            match arg.as_str() {
                "--certificate-json" => output = BelvedereCliOutput::RunCertificateJson,
                _ if arg.starts_with("--") => return Err(BelvedereCliUsageError::new().into()),
                _ => paths.push(arg.clone()),
            }
        }

        let mut paths = paths.into_iter();
        let Some(registry_path) = paths.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        let Some(assessments_path) = paths.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        if paths.next().is_some() {
            return Err(BelvedereCliUsageError::new().into());
        }
        Ok(Self {
            input: BelvedereRunInput::with_invocation(
                registry_path,
                assessments_path,
                BelvedereRunInvocation::cli(arguments),
            ),
            output,
        })
    }

    /// Execute the parsed command.
    ///
    /// # Errors
    ///
    /// Returns an error when artifact collection or any pipeline exchange
    /// rejects its input evidence.
    #[instrument(level = "debug", skip(self))]
    pub fn act(self) -> Result<BelvedereRunReport, BelvedereError> {
        run_belvedere_pipeline(self.input)
    }

    /// Output requested for this command invocation.
    #[must_use]
    pub const fn output(&self) -> BelvedereCliOutput {
        self.output
    }
}

/// Public CLI output contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BelvedereCliOutput {
    /// Log the run summary through the configured tracing subscriber.
    LogSummary,
    /// Write the complete run certificate as JSON.
    RunCertificateJson,
}

/// Error emitted when the Belvedere CLI receives invalid arguments.
#[derive(Debug, Error)]
#[error(
    "usage: belvedere [--certificate-json] <registry-dump.json> <proof-assessments.jsonl> at {file}:{line}"
)]
pub struct BelvedereCliUsageError {
    file: String,
    line: u32,
}

impl BelvedereCliUsageError {
    #[instrument(level = "debug")]
    #[track_caller]
    pub(crate) fn new() -> Self {
        let location = Location::caller();
        Self {
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

/// Error emitted when the CLI cannot write requested output.
#[derive(Debug, Error)]
#[error("Belvedere CLI output failed at {file}:{line}: {kind}")]
pub struct BelvedereCliOutputError {
    #[source]
    kind: Box<BelvedereCliOutputErrorKind>,
    file: String,
    line: u32,
}

impl BelvedereCliOutputError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn serialize(source: serde_json::Error) -> Self {
        Self::new(BelvedereCliOutputErrorKind::Serialize(
            BelvedereCliSerializeError::new(source),
        ))
    }

    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn write(source: io::Error) -> Self {
        Self::new(BelvedereCliOutputErrorKind::Write(
            BelvedereCliWriteError::new(source),
        ))
    }

    #[instrument(level = "debug", skip(kind))]
    #[track_caller]
    fn new(kind: BelvedereCliOutputErrorKind) -> Self {
        let location = Location::caller();
        Self {
            kind: Box::new(kind),
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
enum BelvedereCliOutputErrorKind {
    #[error(transparent)]
    Serialize(BelvedereCliSerializeError),
    #[error(transparent)]
    Write(BelvedereCliWriteError),
}

#[derive(Debug, Error)]
#[error("failed to serialize Belvedere run certificate at {file}:{line}: {source}")]
struct BelvedereCliSerializeError {
    source: serde_json::Error,
    file: String,
    line: u32,
}

impl BelvedereCliSerializeError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn new(source: serde_json::Error) -> Self {
        let location = Location::caller();
        Self {
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

#[derive(Debug, Error)]
#[error("failed to write Belvedere CLI output at {file}:{line}: {source}")]
struct BelvedereCliWriteError {
    source: io::Error,
    file: String,
    line: u32,
}

impl BelvedereCliWriteError {
    #[instrument(level = "debug", skip(source))]
    #[track_caller]
    fn new(source: io::Error) -> Self {
        let location = Location::caller();
        Self {
            source,
            file: location.file().to_owned(),
            line: location.line(),
        }
    }
}

/// Run the Belvedere CLI and return a process exit code.
#[instrument(level = "info", skip(args))]
pub fn run_cli(args: impl IntoIterator<Item = String>) -> ExitCode {
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    run_cli_with_writer(args, &mut writer)
}

/// Run the Belvedere CLI with an explicit output writer.
///
/// This is the testable form of the CLI API. The process entrypoint uses
/// stdout, while tests and embedders may provide any writer they control.
#[instrument(level = "info", skip(args, writer))]
pub fn run_cli_with_writer(args: impl IntoIterator<Item = String>, writer: impl Write) -> ExitCode {
    match run_cli_report(args, writer) {
        Ok(report) => {
            info!(
                claims = report.landscape_receipt().claim_count(),
                proofs = report.landscape_receipt().proof_count(),
                gaps = report.landscape_receipt().gap_count(),
                sections = report.ui_ir_receipt().counts().section_count(),
                trace_widgets = report.ui_ir_receipt().counts().widget_count(),
                tree_nodes = report.ui_ir_receipt().counts().node_count(),
                registry_records = report.registry_receipt().source_record_ids().len(),
                assessment_records = report.assessment_receipt().source_record_ids().len(),
                "belvedere pipeline complete"
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            error!(%error, "belvedere pipeline failed");
            ExitCode::FAILURE
        }
    }
}

#[instrument(level = "debug", skip(args, writer))]
fn run_cli_report(
    args: impl IntoIterator<Item = String>,
    writer: impl Write,
) -> Result<BelvedereRunReport, BelvedereError> {
    let command = BelvedereCli::from_args(args)?;
    let output = command.output();
    let report = command.act()?;
    write_cli_output(output, report.certificate(), writer)?;
    Ok(report)
}

#[instrument(level = "info", skip(output, certificate, writer))]
fn write_cli_output(
    output: BelvedereCliOutput,
    certificate: &BelvedereRunCertificate,
    mut writer: impl Write,
) -> Result<(), BelvedereError> {
    match output {
        BelvedereCliOutput::LogSummary => Ok(()),
        BelvedereCliOutput::RunCertificateJson => {
            serde_json::to_writer_pretty(&mut writer, certificate)
                .map_err(BelvedereCliOutputError::serialize)?;
            writer
                .write_all(b"\n")
                .map_err(BelvedereCliOutputError::write)?;
            Ok(())
        }
    }
}
