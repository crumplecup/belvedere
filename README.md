# Belvedere

Belvedere is a Rust library and CLI for exploring the verification evidence
emitted by Amenable.

Amenable produces certificates, assessments, provenance, and proof records for
formal verification work. Belvedere turns those artifacts into a reviewer-facing
model: a defensive landscape of claims, proofs, gaps, source records, review
work, and UI IR that can be inspected by auditors working on privacy, security,
and safety-critical systems.

The crate is built around explicit contracts. Each pipeline boundary names the
evidence it accepts, the evidence it produces, the receipt it emits, and the
Amenable standards it claims to satisfy. The result is not just a view of a code
base; it is a traceable account of how that view was constructed.

## What Belvedere Produces

Belvedere accepts an Amenable registry dump and a stream of proof assessments,
then builds:

- a `LandscapeGraph` of claims, roots, proofs, assessments, witness surfaces,
  provenance, and gaps;
- an `AuditorNavigationModel` for reviewer workflows such as claim exploration,
  proof-chain deep dives, filter facets, source navigation, assessment queues,
  and review tasks;
- a `UiIrProjection` backed by `elicit_ui`, with traceable sections and widget
  references;
- stage receipts for ingestion, landscape construction, view-model projection,
  and UI IR projection;
- a `BelvedereRunCertificate` and `BelvedereArtifactManifest` that tie the run
  together.

Belvedere can also compare two Amenable artifact sets and produce a certified
landscape diff with reviewer-facing change items, source navigation, receipts,
and a diff-focused UI IR projection.

## Core Concepts

### Contracts

`BelvedereContract` is the shared interface for a named pipeline obligation. A
contract defines:

- `Input`: the evidence accepted at the boundary;
- `Output`: the evidence produced by the boundary;
- `Receipt`: the audit artifact emitted by the boundary;
- `NAME`: a stable contract name;
- `requires()`: preconditions the exchange expects;
- `ensures()`: postconditions the exchange promises;
- `standard_ids()`: Amenable standard identifiers satisfied at that boundary.

The public contracts cover ingestion, landscape modeling, landscape diffs,
auditor navigation, diff review, and `elicit_ui` projection.

### Exchanges

`BelvedereExchange<C>` performs a contract-bound transformation:

```rust
pub trait BelvedereExchange<C>
where
    C: BelvedereContract,
{
    type Error;

    fn exchange(&self, input: C::Input) -> Result<(C::Output, C::Receipt), Self::Error>;
}
```

Concrete exchanges include:

- `RegistryDumpIngestion`
- `AssessmentStreamIngestion`
- `LandscapeModeling`
- `AuditorNavigationProjection`
- `ElicitUiProjection`
- `LandscapeDiffing`
- `LandscapeDiffReviewProjection`
- `ElicitUiDiffReviewProjection`

Each exchange returns both the constructed evidence and a receipt for that
construction step.

### Standards

Belvedere names its own pipeline claims as Amenable standards. The current
catalog includes:

- `InputFidelityStandard`
- `SchemaHonestyStandard`
- `LandscapeConstructionStandard`
- `GapSoundnessStandard`
- `LandscapeDiffFidelityStandard`
- `ViewModelFidelityStandard`
- `UiProjectionFidelityStandard`
- `AccessibilityBaselineStandard`
- `ReportReproducibilityStandard`

Use `belvedere_pipeline_claims()` to enumerate the standards Belvedere exposes
for certificates, reports, and reviewer-facing provenance.

### UI IR

Belvedere projects auditor evidence into `elicit_ui` rather than directly into a
specific frontend toolkit. The projection emits a verified UI tree, stable
section identifiers, traceable widget references, viewport verification evidence,
and minted Amenable standards for the `elicit_ui` properties asserted at the
boundary.

This keeps Belvedere focused on certified view semantics while allowing a
renderer or application shell to decide how the UI is presented.

## Library Usage

Run the normal pipeline over one Amenable registry dump and one proof-assessment
stream:

```rust
use belvedere::{BelvedereRunInput, run_belvedere_pipeline};

fn main() -> Result<(), belvedere::BelvedereError> {
    let report = run_belvedere_pipeline(BelvedereRunInput::new(
        "registry-dump.json",
        "proof-assessments.jsonl",
    ))?;

    let overview = report.view_model().overview();

    println!(
        "{} claims, {} proofs, {} gaps",
        overview.claim_count(),
        overview.proof_coverage_count(),
        overview.gap_count(),
    );

    println!("run certificate: {}", report.certificate().id());
    Ok(())
}
```

Compare two Amenable artifact sets:

```rust
use belvedere::{
    BelvedereLandscapeDiffRunInput, BelvedereRunInput, run_belvedere_landscape_diff,
};

fn main() -> Result<(), belvedere::BelvedereError> {
    let previous = BelvedereRunInput::new(
        "previous-registry-dump.json",
        "previous-proof-assessments.jsonl",
    );
    let current = BelvedereRunInput::new(
        "current-registry-dump.json",
        "current-proof-assessments.jsonl",
    );

    let report = run_belvedere_landscape_diff(BelvedereLandscapeDiffRunInput::new(
        previous,
        current,
    ))?;

    println!(
        "{} changed landscape items",
        report.diff_receipt().counts().total_items(),
    );
    Ok(())
}
```

For lower-level integrations, call individual exchanges directly and carry their
receipts into the next contract boundary.

## CLI Usage

Run Belvedere over a single Amenable artifact set:

```sh
belvedere registry-dump.json proof-assessments.jsonl
```

Compare two artifact sets:

```sh
belvedere diff \
  previous-registry-dump.json previous-proof-assessments.jsonl \
  current-registry-dump.json current-proof-assessments.jsonl
```

The CLI can emit the same artifacts exposed by the library API: certificates,
manifests, ingestion receipts, landscape graphs, auditor navigation models,
review queues, proof-chain details, source navigation, UI IR projections, and
projection receipts.

## Artifact Model

Belvedere keeps the major reviewer surfaces separate:

- `LandscapeGraph` is the source-traceable verification landscape.
- `AuditorNavigationModel` is the UI-neutral reviewer model.
- `UiIrProjection` is the `elicit_ui` representation.
- Receipts explain how each stage was produced.
- Certificates and manifests bind stage evidence into a run-level account.

That separation lets auditors inspect the same run from multiple levels of
detail without losing provenance.

## Error Handling and Telemetry

Public APIs return `BelvedereError` or a more specific exchange error. Errors
preserve source context where Belvedere can identify the failed boundary, such
as artifact collection, ingestion, construction, UI IR projection, or CLI output.

Use `init_tracing()` to install Belvedere's tracing subscriber in binaries that
want structured diagnostics from the pipeline.

## License

Belvedere is licensed under either of:

- Apache License, Version 2.0
- MIT License
