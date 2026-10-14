//! Run invocation provenance contracts.

use derive_getters::Getters;
use serde::Serialize;
use tracing::instrument;

/// Public invocation surface that produced a Belvedere run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BelvedereInvocationSurface {
    /// Belvedere was called through the library API.
    Library,
    /// Belvedere was called through the command-line API.
    Cli,
}

impl BelvedereInvocationSurface {
    /// Stable invocation surface identifier.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Library => "library",
            Self::Cli => "cli",
        }
    }
}

/// Public CLI command contract that produced a Belvedere run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BelvedereInvocationCommand {
    /// Normal landscape navigation run.
    Run,
    /// Certified landscape diff run.
    LandscapeDiff,
}

impl BelvedereInvocationCommand {
    /// Stable command identifier recorded in run certificates.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::LandscapeDiff => "landscape_diff",
        }
    }
}

/// Invocation provenance recorded in a run certificate.
#[derive(Debug, Clone, PartialEq, Eq, Getters, Serialize)]
pub struct BelvedereRunInvocation {
    #[getter(skip)]
    surface: BelvedereInvocationSurface,
    #[getter(skip)]
    command: Option<BelvedereInvocationCommand>,
    output_artifact: Option<String>,
    output_sink: Option<String>,
    arguments: Vec<String>,
}

impl BelvedereRunInvocation {
    /// Invocation provenance for direct library calls.
    #[must_use]
    pub const fn library() -> Self {
        Self {
            surface: BelvedereInvocationSurface::Library,
            command: None,
            output_artifact: None,
            output_sink: None,
            arguments: Vec::new(),
        }
    }

    /// Invocation provenance for command-line calls.
    #[instrument(level = "debug", skip(args))]
    #[must_use]
    pub fn cli(args: impl IntoIterator<Item = String>) -> Self {
        Self {
            surface: BelvedereInvocationSurface::Cli,
            command: None,
            output_artifact: None,
            output_sink: None,
            arguments: args.into_iter().collect(),
        }
    }

    /// Invocation provenance for a normal CLI run.
    #[instrument(level = "debug", skip(args, output_artifact, output_sink))]
    #[must_use]
    pub fn cli_run(
        args: impl IntoIterator<Item = String>,
        output_artifact: Option<String>,
        output_sink: Option<String>,
    ) -> Self {
        Self::cli_command(
            BelvedereInvocationCommand::Run,
            args,
            output_artifact,
            output_sink,
        )
    }

    /// Invocation provenance for a CLI landscape diff run.
    #[instrument(level = "debug", skip(args, output_artifact, output_sink))]
    #[must_use]
    pub fn cli_landscape_diff(
        args: impl IntoIterator<Item = String>,
        output_artifact: Option<String>,
        output_sink: Option<String>,
    ) -> Self {
        Self::cli_command(
            BelvedereInvocationCommand::LandscapeDiff,
            args,
            output_artifact,
            output_sink,
        )
    }

    /// Public surface that produced the run.
    #[must_use]
    pub const fn surface(&self) -> BelvedereInvocationSurface {
        self.surface
    }

    /// Public command contract that produced the run, when applicable.
    #[must_use]
    pub const fn command(&self) -> Option<BelvedereInvocationCommand> {
        self.command
    }

    #[instrument(level = "debug", skip(command, args))]
    fn cli_command(
        command: BelvedereInvocationCommand,
        args: impl IntoIterator<Item = String>,
        output_artifact: Option<String>,
        output_sink: Option<String>,
    ) -> Self {
        Self {
            surface: BelvedereInvocationSurface::Cli,
            command: Some(command),
            output_artifact,
            output_sink,
            arguments: args.into_iter().collect(),
        }
    }
}
