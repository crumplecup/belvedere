//! Command-line entrypoints for Belvedere.

use std::{
    fs::File,
    io::{self, Write},
    panic::Location,
    path::PathBuf,
    process::ExitCode,
};

use serde::Serialize;
use thiserror::Error;
use tracing::{error, info, instrument};

use crate::{
    BelvedereError, BelvedereLandscapeDiffRunInput, BelvedereLandscapeDiffRunReport,
    BelvedereRunInput, BelvedereRunInvocation, BelvedereRunReport, run_belvedere_landscape_diff,
    run_belvedere_pipeline,
};

/// Parsed Belvedere command-line invocation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BelvedereCli {
    command: BelvedereCliCommand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BelvedereCliCommand {
    Help,
    Run {
        input: BelvedereRunInput,
        output: BelvedereCliOutput,
    },
    Diff {
        input: BelvedereLandscapeDiffRunInput,
        output: BelvedereCliOutput,
    },
}

/// Stable CLI help text.
pub const BELVEDERE_CLI_HELP: &str = "\
usage: belvedere [--help] [--certificate-json | --certificate-file <path>] <registry-dump.json> <proof-assessments.jsonl>
       belvedere diff [--certificate-json | --certificate-file <path> | --diff-json] <previous-registry-dump.json> <previous-proof-assessments.jsonl> <current-registry-dump.json> <current-proof-assessments.jsonl>

Options:
  -h, --help                 Show this help text.
      --certificate-json     Write the requested certificate as JSON to stdout.
      --certificate-file     Write the requested certificate as JSON to a file.
      --diff-json            Write the landscape diff as JSON to stdout.
";

impl BelvedereCli {
    /// Parse a Belvedere pipeline run from positional artifact paths.
    ///
    /// # Errors
    ///
    /// Returns a usage error unless exactly two artifact paths are supplied.
    #[instrument(level = "debug", skip(args))]
    pub fn from_args(args: impl IntoIterator<Item = String>) -> Result<Self, BelvedereError> {
        let arguments = args.into_iter().collect::<Vec<_>>();
        if arguments
            .iter()
            .any(|argument| matches!(argument.as_str(), "-h" | "--help"))
        {
            return Ok(Self {
                command: BelvedereCliCommand::Help,
            });
        }
        if arguments.first().is_some_and(|argument| argument == "diff") {
            return Self::from_diff_args(arguments);
        }

        Self::from_run_args(arguments)
    }

    #[instrument(level = "debug", skip(arguments))]
    fn from_run_args(arguments: Vec<String>) -> Result<Self, BelvedereError> {
        let mut output = BelvedereCliOutput::LogSummary;
        let mut paths = Vec::new();
        let mut index = 0;

        while let Some(arg) = arguments.get(index) {
            match arg.as_str() {
                "--certificate-json" => output = BelvedereCliOutput::RunCertificateJson,
                "--certificate-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunCertificateFile(PathBuf::from(path));
                    index += 1;
                }
                "--diff-json" => return Err(BelvedereCliUsageError::new().into()),
                _ if arg.starts_with("--") => return Err(BelvedereCliUsageError::new().into()),
                _ => paths.push(arg.clone()),
            }
            index += 1;
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
            command: BelvedereCliCommand::Run {
                input: BelvedereRunInput::with_invocation(
                    registry_path,
                    assessments_path,
                    BelvedereRunInvocation::cli(arguments),
                ),
                output,
            },
        })
    }

    #[instrument(level = "debug", skip(arguments))]
    fn from_diff_args(arguments: Vec<String>) -> Result<Self, BelvedereError> {
        let mut output = BelvedereCliOutput::LogSummary;
        let mut paths = Vec::new();
        let mut index = 1;

        while let Some(arg) = arguments.get(index) {
            match arg.as_str() {
                "--certificate-json" => {
                    output = BelvedereCliOutput::LandscapeDiffCertificateJson;
                }
                "--certificate-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::LandscapeDiffCertificateFile(PathBuf::from(path));
                    index += 1;
                }
                "--diff-json" => output = BelvedereCliOutput::LandscapeDiffJson,
                _ if arg.starts_with("--") => return Err(BelvedereCliUsageError::new().into()),
                _ => paths.push(arg.clone()),
            }
            index += 1;
        }

        let mut paths = paths.into_iter();
        let Some(previous_registry_path) = paths.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        let Some(previous_assessments_path) = paths.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        let Some(current_registry_path) = paths.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        let Some(current_assessments_path) = paths.next() else {
            return Err(BelvedereCliUsageError::new().into());
        };
        if paths.next().is_some() {
            return Err(BelvedereCliUsageError::new().into());
        }

        let invocation = BelvedereRunInvocation::cli(arguments);
        Ok(Self {
            command: BelvedereCliCommand::Diff {
                input: BelvedereLandscapeDiffRunInput::new(
                    BelvedereRunInput::with_invocation(
                        previous_registry_path,
                        previous_assessments_path,
                        invocation.clone(),
                    ),
                    BelvedereRunInput::with_invocation(
                        current_registry_path,
                        current_assessments_path,
                        invocation,
                    ),
                ),
                output,
            },
        })
    }

    /// Execute the parsed command.
    ///
    /// # Errors
    ///
    /// Returns an error when artifact collection or any pipeline exchange
    /// rejects its input evidence. Help invocations return no run report.
    #[instrument(level = "debug", skip(self))]
    pub fn act(self) -> Result<Option<BelvedereCliReport>, BelvedereError> {
        match self.command {
            BelvedereCliCommand::Help => Ok(None),
            BelvedereCliCommand::Run { input, .. } => run_belvedere_pipeline(input)
                .map(Box::new)
                .map(BelvedereCliReport::Run)
                .map(Some),
            BelvedereCliCommand::Diff { input, .. } => run_belvedere_landscape_diff(input)
                .map(Box::new)
                .map(BelvedereCliReport::LandscapeDiff)
                .map(Some),
        }
    }

    /// Output requested for this command invocation.
    #[must_use]
    #[instrument(level = "trace", skip(self))]
    pub fn output(&self) -> BelvedereCliOutput {
        match &self.command {
            BelvedereCliCommand::Help => BelvedereCliOutput::Help,
            BelvedereCliCommand::Run { output, .. } | BelvedereCliCommand::Diff { output, .. } => {
                output.clone()
            }
        }
    }
}

/// Report emitted by a successful CLI action.
#[derive(Debug, Clone)]
pub enum BelvedereCliReport {
    /// A normal Belvedere pipeline report.
    Run(Box<BelvedereRunReport>),
    /// A certified landscape diff report.
    LandscapeDiff(Box<BelvedereLandscapeDiffRunReport>),
}

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
}

/// Error emitted when the Belvedere CLI receives invalid arguments.
#[derive(Debug, Error)]
#[error("invalid Belvedere CLI arguments at {file}:{line}: {usage}")]
pub struct BelvedereCliUsageError {
    usage: String,
    file: String,
    line: u32,
}

impl BelvedereCliUsageError {
    #[instrument(level = "debug")]
    #[track_caller]
    pub(crate) fn new() -> Self {
        let location = Location::caller();
        Self {
            usage: BELVEDERE_CLI_HELP.to_owned(),
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
#[error("failed to serialize Belvedere CLI JSON output at {file}:{line}: {source}")]
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
        Ok(Some(BelvedereCliReport::Run(report))) => {
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
        Ok(Some(BelvedereCliReport::LandscapeDiff(report))) => {
            info!(
                added = report.diff_receipt().counts().added_items(),
                removed = report.diff_receipt().counts().removed_items(),
                changed = report.diff_receipt().counts().changed_items(),
                total = report.diff_receipt().counts().total_items(),
                source_records = report.diff_receipt().source_record_ids().len(),
                "belvedere landscape diff complete"
            );
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
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
) -> Result<Option<BelvedereCliReport>, BelvedereError> {
    let command = BelvedereCli::from_args(args)?;
    let output = command.output();
    if output == BelvedereCliOutput::Help {
        write_cli_help(writer)?;
        return Ok(None);
    }

    let Some(report) = command.act()? else {
        return Err(BelvedereCliUsageError::new().into());
    };
    write_cli_output(output, &report, writer)?;
    Ok(Some(report))
}

#[instrument(level = "info", skip(output, report, writer))]
fn write_cli_output(
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
        (BelvedereCliOutput::LandscapeDiffJson, BelvedereCliReport::LandscapeDiff(diff)) => {
            write_json(diff.diff(), writer)
        }
        _ => Err(BelvedereCliUsageError::new().into()),
    }
}

#[instrument(level = "info", skip(writer))]
fn write_cli_help(mut writer: impl Write) -> Result<(), BelvedereError> {
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
