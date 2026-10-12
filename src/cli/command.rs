//! CLI command parsing and execution.

mod diff_output;
mod output_parts;
mod run_output;

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
        let mut artifact = None;
        let mut sink = None;
        let mut paths = Vec::new();
        let mut index = 0;

        while let Some(arg) = arguments.get(index) {
            if let Some(consumed) =
                Self::parse_run_output_arg(&arguments, index, &mut artifact, &mut sink)?
            {
                index += consumed;
            } else if arg.starts_with("--") {
                return Err(BelvedereCliUsageError::new().into());
            } else {
                paths.push(arg.clone());
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
                output: Self::output_from_parts(artifact, sink)?,
            },
        })
    }

    #[instrument(level = "debug", skip(arguments))]
    fn from_diff_args(arguments: Vec<String>) -> Result<Self, BelvedereError> {
        let mut artifact = None;
        let mut sink = None;
        let mut paths = Vec::new();
        let mut index = 1;

        while let Some(arg) = arguments.get(index) {
            if let Some(consumed) =
                Self::parse_diff_output_arg(&arguments, index, &mut artifact, &mut sink)?
            {
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
                output: Self::output_from_parts(artifact, sink)?,
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
