//! CLI command parsing and execution.

mod diff_output;

use std::path::PathBuf;

use tracing::instrument;

use crate::{
    BelvedereError, BelvedereLandscapeDiffRunInput, BelvedereRunInput, BelvedereRunInvocation,
    run_belvedere_landscape_diff, run_belvedere_pipeline,
};

use super::{BelvedereCliOutput, BelvedereCliReport, BelvedereCliUsageError};

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
                "--manifest-json" => output = BelvedereCliOutput::RunManifestJson,
                "--manifest-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunManifestFile(PathBuf::from(path));
                    index += 1;
                }
                "--registry-receipt-json" => {
                    output = BelvedereCliOutput::RunRegistryReceiptJson;
                }
                "--registry-receipt-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunRegistryReceiptFile(PathBuf::from(path));
                    index += 1;
                }
                "--assessment-receipt-json" => {
                    output = BelvedereCliOutput::RunAssessmentReceiptJson;
                }
                "--assessment-receipt-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunAssessmentReceiptFile(PathBuf::from(path));
                    index += 1;
                }
                "--landscape-json" => {
                    output = BelvedereCliOutput::RunLandscapeJson;
                }
                "--landscape-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunLandscapeFile(PathBuf::from(path));
                    index += 1;
                }
                "--landscape-receipt-json" => {
                    output = BelvedereCliOutput::RunLandscapeReceiptJson;
                }
                "--landscape-receipt-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunLandscapeReceiptFile(PathBuf::from(path));
                    index += 1;
                }
                "--view-model-json" => {
                    output = BelvedereCliOutput::RunViewModelJson;
                }
                "--view-model-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunViewModelFile(PathBuf::from(path));
                    index += 1;
                }
                "--claim-details-json" => {
                    output = BelvedereCliOutput::RunClaimDetailsJson;
                }
                "--claim-details-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunClaimDetailsFile(PathBuf::from(path));
                    index += 1;
                }
                "--filter-facets-json" => {
                    output = BelvedereCliOutput::RunFilterFacetsJson;
                }
                "--filter-facets-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunFilterFacetsFile(PathBuf::from(path));
                    index += 1;
                }
                "--source-navigation-json" => {
                    output = BelvedereCliOutput::RunSourceNavigationJson;
                }
                "--source-navigation-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunSourceNavigationFile(PathBuf::from(path));
                    index += 1;
                }
                "--proof-chains-json" => {
                    output = BelvedereCliOutput::RunProofChainsJson;
                }
                "--proof-chains-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunProofChainsFile(PathBuf::from(path));
                    index += 1;
                }
                "--review-workflow-json" => {
                    output = BelvedereCliOutput::RunReviewWorkflowJson;
                }
                "--review-workflow-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunReviewWorkflowFile(PathBuf::from(path));
                    index += 1;
                }
                "--view-model-receipt-json" => {
                    output = BelvedereCliOutput::RunViewModelReceiptJson;
                }
                "--view-model-receipt-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunViewModelReceiptFile(PathBuf::from(path));
                    index += 1;
                }
                "--ui-ir-json" => output = BelvedereCliOutput::RunUiIrJson,
                "--ui-ir-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunUiIrFile(PathBuf::from(path));
                    index += 1;
                }
                "--ui-ir-receipt-json" => output = BelvedereCliOutput::RunUiIrReceiptJson,
                "--ui-ir-receipt-file" => {
                    let Some(path) = arguments.get(index + 1) else {
                        return Err(BelvedereCliUsageError::new().into());
                    };
                    output = BelvedereCliOutput::RunUiIrReceiptFile(PathBuf::from(path));
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
            if let Some(consumed) = Self::parse_diff_output_arg(&arguments, index, &mut output)? {
                index += consumed;
            } else if arg.starts_with("--") {
                return Err(BelvedereCliUsageError::new().into());
            } else {
                paths.push(arg.clone());
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

    #[instrument(level = "debug")]
    pub(in crate::cli::command) fn next_path(
        arguments: &[String],
        index: usize,
    ) -> Result<PathBuf, BelvedereError> {
        let Some(path) = arguments.get(index + 1) else {
            return Err(BelvedereCliUsageError::new().into());
        };
        Ok(PathBuf::from(path))
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
