//! CLI command parsing and execution.

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

    #[instrument(level = "info", skip(output))]
    fn parse_diff_output_arg(
        arguments: &[String],
        index: usize,
        output: &mut BelvedereCliOutput,
    ) -> Result<Option<usize>, BelvedereError> {
        let consumed = match arguments[index].as_str() {
            "--certificate-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCertificateJson;
                0
            }
            "--certificate-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCertificateFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--run-certificate-json" => {
                *output = BelvedereCliOutput::LandscapeDiffRunCertificateJson;
                0
            }
            "--run-certificate-file" => {
                *output = BelvedereCliOutput::LandscapeDiffRunCertificateFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--diff-json" => {
                *output = BelvedereCliOutput::LandscapeDiffJson;
                0
            }
            "--diff-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffReceiptJson;
                0
            }
            "--diff-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--previous-registry-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptJson;
                0
            }
            "--previous-registry-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousRegistryReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-assessment-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptJson;
                0
            }
            "--previous-assessment-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousAssessmentReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-registry-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptJson;
                0
            }
            "--current-registry-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentRegistryReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-assessment-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptJson;
                0
            }
            "--current-assessment-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentAssessmentReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--previous-landscape-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptJson;
                0
            }
            "--previous-landscape-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffPreviousLandscapeReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-landscape-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptJson;
                0
            }
            "--current-landscape-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentLandscapeReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-view-model-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelJson;
                0
            }
            "--current-view-model-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--current-view-model-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptJson;
                0
            }
            "--current-view-model-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentViewModelReceiptFile(
                    Self::next_path(arguments, index)?,
                );
                1
            }
            "--current-ui-ir-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrJson;
                0
            }
            "--current-ui-ir-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--current-ui-ir-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptJson;
                0
            }
            "--current-ui-ir-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffCurrentUiIrReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--landscape-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffLandscapeReceiptJson;
                0
            }
            "--landscape-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffLandscapeReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--view-model-json" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelJson;
                0
            }
            "--view-model-file" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--view-model-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelReceiptJson;
                0
            }
            "--view-model-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffViewModelReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            "--ui-ir-json" => {
                *output = BelvedereCliOutput::LandscapeDiffUiIrJson;
                0
            }
            "--ui-ir-file" => {
                *output =
                    BelvedereCliOutput::LandscapeDiffUiIrFile(Self::next_path(arguments, index)?);
                1
            }
            "--ui-ir-receipt-json" => {
                *output = BelvedereCliOutput::LandscapeDiffUiIrReceiptJson;
                0
            }
            "--ui-ir-receipt-file" => {
                *output = BelvedereCliOutput::LandscapeDiffUiIrReceiptFile(Self::next_path(
                    arguments, index,
                )?);
                1
            }
            _ => return Ok(None),
        };

        Ok(Some(consumed))
    }

    #[instrument(level = "debug")]
    fn next_path(arguments: &[String], index: usize) -> Result<PathBuf, BelvedereError> {
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
