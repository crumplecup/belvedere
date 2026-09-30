# Belvedere Strategic Plan

Belvedere is the view layer for systems whose defensive posture is described by
Amenable. Its job is to help a user inspect the landscape they have built:
certificates, provenance, proof chains, proof assessments, backend coverage,
witness surfaces, and the boundaries where trust remains explicit.

The initial target is not a native GUI or web frontend. Belvedere should first
produce a UI intermediate representation using `elicit_ui`, the local
proof-carrying UI interface crate from Elicitation. That choice matters: if
Amenable makes verification evidence auditable, Belvedere should make its own
presentation surface auditable too.

## North Star

Belvedere should make a verified codebase feel navigable.

The core user experience is a defensive landscape view: a structured map of
claims, roots, proofs, reviewer assessments, and unresolved gaps. A reviewer
should be able to answer, quickly and with confidence:

- What claims does this system depend on?
- Which claims are proven, which are certified, and which are still gaps?
- Which backend verified each claim: Kani, Creusot, Verus, or some subset?
- Which assumptions are roots rather than derived facts?
- Which proof harnesses have human assessments, and what did reviewers say?
- Where does a claim's proof chain branch, converge, or stop?
- Which parts of the landscape are ready for safety/security review, and which
  still need strengthening, replacement, or retirement?

Belvedere should not hide uncertainty. Its value is not a green dashboard. Its
value is an honest, beautiful, reviewable map.

## Source Systems

This plan is grounded in the current local Amenable and Elicitation surfaces:

- `amenable` re-exports the core constitutional roles and provides CLI
  inspection commands such as `audit`, `assess`, `dump-registry`, `gallery`,
  and `verify`.
- `amenable_core` exposes the core evidence/proof graph: `EvidenceLink`,
  `ProofRecord`, `ContractRecord`, `ExchangeEdgeRecord`,
  `ProofTokenMintRecord`, `WitnessExportRecord`, `WitnessExportSnapshot`, and
  proof-chain reconstruction through `proof_chain` and
  `proof_chain_for_verifiers`.
- Amenable's registry dump shape includes `evidence_links`, `proof_records`,
  `contract_records`, `witness_export_records`, and `kani_proofs`.
- Amenable's assessment stream is persisted as JSON Lines with proof IDs,
  reviewers, timestamps, rubric scores, recommendations, resolution paths, and
  comments.
- `elicit_ui` models UI through AccessKit-backed verified trees, WCAG proof
  sidecars, layout/navigation/event/inspection traits, and renderer traits that
  translate AccessKit roles into frontend output.

The first Belvedere implementation should prefer these published surfaces over
scraping rendered CLI text. Rendered audit text is useful for display; registry
records and assessment JSON are the correct ingestion contracts.

## Product Vocabulary

Belvedere needs its own small domain model between Amenable and `elicit_ui`.
This is not a second proof system. It is a projection layer for review.

Core concepts:

- `Landscape`: one inspected project or artifact set, including registry,
  assessments, generated proofs, and source metadata.
- `BelvedereContract`: a named obligation for one pipeline boundary, with
  explicit input shape, output shape, preconditions, postconditions, and audit
  vocabulary.
- `BelvedereExchange`: a proof-bearing transformation from one certified
  pipeline state to the next.
- `Claim`: a named evidence/proposition node.
- `Root`: a claim whose basis is itself and whose support comes from explicit
  provenance/certification rather than a prior proof edge.
- `Proof`: a backend-specific proof record for a claim.
- `BackendCoverage`: the verifier set attached to a claim or chain.
- `Assessment`: a human review event over a proof or proof harness.
- `Recommendation`: accept, strengthen, replace, or retire.
- `ResolutionPath`: the concrete work implied by a recommendation.
- `WitnessSurface`: a structured witness export tree with support summaries.
- `Gap`: an absent proof, unsupported backend, stale assessment, low rubric
  dimension, missing provenance, or incomplete chain.
- `ViewModel`: a stable, serializable Belvedere projection ready to render.

This vocabulary should remain UI-neutral. It should be testable without any
renderer and rich enough to support multiple presentations later.

## Architecture

Belvedere should grow in four phases, but each phase should be designed first
as an Amenable interface rather than as ordinary application plumbing. The
viewer work is ultimately straightforward once the semantic boundaries are
right. The important early design task is naming the contracts and exchanges
that govern each segment of the pipeline.

Every phase should declare:

- the input evidence it accepts
- the output evidence it promises
- the standards or provenance roots it depends on
- the `Requires` and `Ensures` obligations that describe the exchange
- the certificate or receipt it emits for auditors
- the assessments humans may need to perform when proof is unavailable or
  judgment-dependent

The implementation behind a phase may be simple Rust code. The public design
surface should still be Amenable-shaped: named evidence, explicit roots,
contracts, exchanges, certificates, proofs, and assessments.

### 1. Collection

Collection is the first contract/exchange boundary. It reads Amenable outputs
and local files into typed snapshots, but its real role is to define what it
means for an artifact to become admissible Belvedere evidence.

Initial collectors:

- Registry dump collector: reads the JSON shape produced by Amenable's
  `dump-registry`.
- Assessment collector: reads Amenable's proof assessment JSON Lines.
- Proof-chain collector: calls `amenable::proof_chain_for_verifiers` when
  linked records are available in-process.
- Artifact collector: reads certificate and proof artifact directories only
  when there is a stable file shape worth modeling.

The collector layer should preserve raw IDs and source paths. It should avoid
normalizing away the details a reviewer may need to trace.

The first ingestion exchange should be named before it is made clever. A good
initial shape:

- `RawAmenableArtifact`: evidence for an input file plus path, digest, and
  declared artifact kind.
- `AcceptedAmenableArtifactFormat`: a standard root for the schema Belvedere
  agrees to understand, such as registry dump JSON or assessment JSON Lines.
- `DecodedRegistrySnapshot`: evidence for decoded registry records with every
  source identity preserved.
- `DecodedAssessmentStream`: evidence for decoded assessment events with proof
  IDs, reviewers, timestamps, recommendations, resolution paths, and comments
  preserved.
- `IngestionReceipt`: the audit surface for the exchange: input digest,
  artifact kind, schema root, parser version, decoded counts, unsupported
  fields, warnings, and source-record IDs.

The exchange contract should say, in Amenable terms: given a raw artifact
certified against an accepted format, Belvedere may produce a decoded snapshot
only if it can account for record identity, artifact provenance, unsupported
data, and any lossy or uncertain interpretation. The first proof target is
not "the UI is correct"; it is "ingestion did not silently erase the evidence
surface an auditor needs."

### 2. Landscape Model

The model phase is the next exchange. It turns decoded records into a landscape
graph, but the exchange contract should be explicit about which records become
claims, edges, roots, proofs, contracts, witness surfaces, assessments, and
gaps.

- evidence links become directed edges from claim to basis
- proof records attach verifier coverage to claims
- contract records attach backend-native predicates and contract kind
- witness exports attach structural proof-support summaries
- Kani proof registrations attach harness IDs and packages
- assessments attach review history and current recommendation status

The model should compute derived review facts:

- root claims
- fan-out/fan-in points
- backend completeness by claim and by chain
- stale or superseded assessment sequences
- claims with proof records but no assessment
- assessments that recommend replacement or retirement
- witness support distribution: checked, trusted, trivial, opaque

Its contract should favor conservative incompleteness: a landscape may expose
extra gaps or unknowns, but it must not certify completeness for an absent
proof, incomplete chain, unsupported backend, or stale assessment.

### 3. View Model

The view-model phase exchanges a landscape certificate for auditor navigation
evidence. It translates the landscape graph into task-oriented screens while
remaining independent of any visual renderer.

Initial views:

- Overview: totals, verifier coverage, high-priority gaps, recent assessment
  status.
- Claim Explorer: searchable evidence graph with selected claim details.
- Chain View: one subject's proof tree, roots, backend proofs, and gaps.
- Assessment Queue: proofs grouped by recommendation and rubric weakness.
- Witness Surface: tree view of witness exports and support summaries.
- Provenance Roots: all explicit roots and the metadata that grounds them.

This is where Belvedere should decide what is important, not where a frontend
should reinvent interpretation.

Its contract should require traceability: every visible assertion, count,
status, badge, queue entry, and navigation target must retain a source path
back to the landscape evidence that produced it.

### 4. `elicit_ui` Projection

The UI projection phase exchanges auditor navigation evidence for `elicit_ui`
IR evidence. It maps each view model into an `elicit_ui` layout.

Early projection should use boring, accessible primitives first:

- headings for view and region titles
- navigation landmarks for major sections
- lists and tables for claims, proofs, and assessments
- links for source/artifact references
- buttons for explicit commands
- progress indicators for backend coverage summaries
- labels/status text for recommendation, verifier, and gap state

Belvedere should verify generated layouts with `elicit_ui` before any renderer
consumes them. The renderer-independent acceptance criterion is a verified
AccessKit-backed tree that faithfully represents the landscape.
The exchange contract should require semantic preservation: roles, labels,
relationships, status text, and source IDs cannot disappear merely because the
output is now UI IR.

## Product Build Plan

Belvedere should be built as a full certification navigator, not as a demo or
trial of the Amenable pipeline. The framework is already the trusted foundation.
The phases below are product construction order: define the semantic contract
first, then implement the product surface that depends on it.

1. Add a library crate surface beside the existing binary.
2. Add dependencies on `serde`, `serde_json`, `thiserror` or the repository's
   chosen error stack, and the Amenable crate already available locally.
3. Define the accepted ingestion standards: registry dump JSON and assessment
   JSON Lines.
4. Define the first evidence types: raw artifact, decoded registry snapshot,
   decoded assessment stream, and ingestion receipt.
5. Define the ingestion contracts: required artifact metadata, accepted schema
   roots, identity preservation, unsupported-field handling, and loss reporting.
6. Define the ingestion exchanges from raw artifact evidence to decoded
   snapshot evidence.
7. Implement the parser and collector code behind those exchanges.
8. Test the exchange with representative fixtures and assert that record IDs,
   source paths, verifier names, proof IDs, recommendations, and unsupported
   data remain accountable.
9. Emit an ingestion receipt/certificate that Amenable can later register and
   Belvedere can later display.
10. Continue through the remaining phases with the same structure: landscape
    contracts and exchanges, auditor navigation view models, `elicit_ui`
    projection, stage certificates, and reviewer-facing workflows.

This is not a viability test. It is the first product increment because
ingestion is the first place where auditor-facing trust can be preserved or
lost. The delivery bar is not "a small slice works"; it is that the first
segment of a production navigator has the same explicit contract/exchange
shape as the rest of the product.

## Design Principles

- Truth before beauty: the view must preserve proof boundaries, uncertainty,
  and provenance roots.
- Projection before rendering: Belvedere owns the review model; renderers only
  display it.
- Read-only first: early Belvedere should inspect and explain before it mutates
  assessments, proof queues, or source artifacts.
- Stable IDs everywhere: claim names, proof IDs, verifier names, and artifact
  paths should survive every projection.
- Explicit incompleteness: missing proofs and reviewer concerns are first-class
  objects, not warnings hidden in logs.
- Accessibility by construction: `elicit_ui` verification is part of the main
  path, not a later audit.
- Backend honesty: do not imply parity when Kani, Creusot, and Verus have
  different available records or generated artifacts.
- Reviewer empathy: every screen should answer the next review question rather
  than merely display all data.
- Ordinary inputs only: Belvedere's own repository may be used as a validation
  corpus, but it must enter through the same Amenable artifact path as any
  other project.

## Making Belvedere Amenable

Belvedere should not merely visualize Amenable outputs. Its own pipeline should
also be written in a way Amenable can evaluate: named contracts, explicit
roots, generated certificates, machine-checkable proofs where practical, and
human assessments where judgment is still required.

That gives the project a useful acceptance scenario:

1. Amenable describes a repository's defensive posture.
2. Belvedere ingests that Amenable output.
3. Belvedere renders a landscape for that repository.
4. Reviewers use the landscape to inspect ingestion, modeling, projection,
   accessibility, and proof coverage.

Belvedere's repository can be one such repository, but it should not receive a
special contract family, exchange, certificate, receipt, or view model. The
authority remains the underlying Amenable records, backend verifier results,
explicit provenance roots, and reviewer assessments.

### Pipeline Claims

The first Belvedere claims should track the project-neutral pipeline boundaries
that matter most:

- Input fidelity: a registry dump or assessment log is parsed without dropping
  record identity, source path, verifier name, recommendation, or provenance
  metadata.
- Schema honesty: unknown or unsupported Amenable fields remain visible as
  unsupported data rather than silently disappearing.
- Landscape construction: every modeled claim, proof, root, assessment, and
  gap is traceable to one or more collected input records.
- Gap soundness: Belvedere may over-report uncertainty, but it must not mark an
  absent proof, incomplete chain, stale assessment, or unsupported backend as
  complete.
- View-model fidelity: each visible row, badge, counter, or navigation target
  has a stable source ID back to the landscape model.
- UI projection fidelity: each view model projects into `elicit_ui` without
  erasing semantic roles, labels, relationships, or status text.
- Accessibility baseline: emitted UI IR satisfies the available `elicit_ui`
  accessibility and verified-tree checks.
- Report reproducibility: the same input snapshot produces the same landscape
  and view model output, modulo explicit timestamps and tool versions.

These should become named Amenable claims early, even before every claim has a
machine proof. Naming the claims creates stable handles for certificates,
assessments, and future proofs.

### Evidence Emitted By Each Stage

Each Belvedere layer should emit or expose evidence that Amenable can register:

- Collection should produce a `CollectorReceipt`: input path, content digest,
  parser version, decoded record counts, skipped/unsupported fields, and any
  recoverable warnings.
- Landscape construction should produce a `LandscapeCertificate`: source
  receipt IDs, graph node and edge counts, gap counts, root counts, and
  invariant check results.
- View-model construction should produce a `ViewModelCertificate`: source
  landscape ID, view kind, stable IDs included, summaries computed, and
  invariant check results.
- UI projection should produce a `UiProjectionCertificate`: source view model
  ID, `elicit_ui` verification result, widget IDs emitted, accessibility
  checks, and any renderer-independent warnings.
- A full Belvedere run should produce a `BelvedereRunCertificate`: Belvedere
  version, Amenable version, Elicitation version, input artifact digests,
  generated certificate digests, and the command line used.

These certificates do not need to be visually fancy. They need to be structured,
deterministic, and boring enough that Amenable can treat them as evidence
records.

### Standards, Proofs, And Assessments

Belvedere's pipeline evidence should use the same trust vocabulary it displays
for every project:

- Standards are explicit roots for things that are certified rather than
  derived, such as the accepted registry-dump schema, assessment-log schema,
  and `elicit_ui` interface contract.
- Proofs should start with pure, low-I/O logic: parsers, graph construction,
  coverage classification, latest-assessment selection, gap detection, stable
  ordering, and view-model projection.
- Backend coverage should grow incrementally. Kani is a natural first target
  for bounded Rust behavior; Creusot and Verus can follow where specifications
  and translation support make sense.
- Assessments should cover human judgment: whether the visual vocabulary is
  honest, whether labels communicate uncertainty, whether a reviewer can trace
  a displayed assertion back to its source, and whether the landscape helps an
  auditor act instead of merely looking complete.

This keeps the promise crisp: Belvedere can prove and certify properties of
its pipeline, while still making human review explicit for product and
communication quality.

### Repository Fixture Workflow

A mature local gate should eventually look like this:

1. Build Belvedere and run focused tests.
2. Generate or load Amenable registry, proof, certificate, and assessment
   artifacts for one or more representative repositories.
3. Run Belvedere against those artifacts to create landscape snapshots.
4. Verify the view models and `elicit_ui` layouts.
5. Publish or store the resulting stage certificates under the configured
   artifact location.
6. Fail the gate on parser loss, invalid landscape invariants, broken UI IR,
   missing source traceability, or open high-severity findings.

Belvedere's own repository can be included in that fixture set because it is a
real Amenable codebase. It should be treated exactly like the others.

## Early Data Questions

These are the main questions to resolve during implementation:

- Should Belvedere ingest Amenable by linking the `amenable` crate directly,
  reading `dump-registry` JSON, or supporting both?
- Which registry dump fields should become a semver-stable Belvedere input
  contract?
- Should assessment history be collapsed to latest-by-proof by default, with a
  detail view for prior reviews?
- How should Belvedere represent proofs that timed out, failed due to backend
  limitations, or are intentionally scoped?
- What is the minimum visual vocabulary for proof confidence without implying
  a single misleading score?
- Which view model invariants should get their own tests before any renderer
  exists?
- Which Belvedere pipeline claims should become Amenable records in the first
  certification pass?
- What is the smallest certificate shape that preserves source traceability
  without turning every view computation into a heavyweight proof artifact?
- Which roots should be explicit Belvedere standards rather than machine
  proofs: schema acceptance, UI vocabulary, reviewer workflow, or renderer
  expectations?

## Deferred Product Scope

- No custom GUI renderer.
- No web app or native app shell.
- No live Kani, Creusot, or Verus execution.
- No assessment editing workflow.
- No attempt to replace Amenable's audit or registry logic.
- No graph-layout cleverness until the contract/exchange boundaries and typed
  landscape model are correct.
- No claim that Belvedere validates itself merely because it can render its own
  Amenable output.

## Product Validation Strategy

Belvedere's checks should be executable product gates:

- define and compile the ingestion evidence, standard, contract, receipt, and
  exchange surfaces
- parse representative registry dump JSON
- parse representative assessment JSON Lines
- emit ingestion receipts that account for source paths, digests, decoded
  counts, unsupported fields, warnings, and source-record IDs
- construct a landscape graph with roots, proofs, contracts, and assessments
- compute backend coverage and assessment recommendations deterministically
- project at least one overview and one claim detail into `elicit_ui`
- verify the resulting layout at multiple viewports
- emit a certificate for each successful collection, landscape, view-model, and
  UI projection step
- run fixtures where Belvedere displays Amenable evidence for representative
  repositories, including this repository when useful
- run `cargo fmt`, focused tests, and the repository's cordial gate before
  calling implementation work complete

Once real code exists, every new view should have tests at both levels: the
view model it intends to communicate and the `elicit_ui` layout it emits.
Fixture-facing tests should also assert that visible claims retain source IDs
back to the collected evidence that produced them.

## Suggested Repository Shape

```text
src/
  lib.rs
  main.rs
  contracts/
    mod.rs
    ingestion.rs
    landscape.rs
    view_model.rs
    ui_projection.rs
  exchanges/
    mod.rs
    ingestion.rs
    landscape.rs
    view_model.rs
    ui_projection.rs
  collect/
    mod.rs
    registry_dump.rs
    assessments.rs
  landscape/
    mod.rs
    graph.rs
    coverage.rs
    assessment_status.rs
  view_model/
    mod.rs
    overview.rs
    claim.rs
  ui/
    mod.rs
    overview.rs
    claim.rs
  certify/
    mod.rs
    receipt.rs
    landscape.rs
    ui_projection.rs
tests/
  fixtures/
    registry_dump.json
    proof_assessments.jsonl
    belvedere_registry_dump.json
  ingestion_exchange_test.rs
  landscape_snapshot_test.rs
  ui_projection_test.rs
  repository_fixture_snapshot_test.rs
```

This shape keeps the Amenable-facing interfaces visible instead of burying them
inside parsers or renderers. The `contracts/` and `exchanges/` modules name the
semantic promises. The `collect/`, `landscape/`, `view_model/`, and `ui/`
modules implement those promises. That separation will matter as soon as
Belvedere supports more than one presentation or more than one Amenable
artifact source.

## Product Direction

Belvedere should grow into a review cockpit:

- interactive filtering by verifier, recommendation, support kind, package, or
  claim family
- focused chain inspection for one claim
- diffing two landscape snapshots
- queue views for replacement/strengthening work
- provenance catalogs for explicit trust roots
- source navigation into proof harnesses and generated companions
- renderer crates for terminal, web, or desktop surfaces
- eventually, carefully gated assessment-writing workflows

The important discipline is that each feature should make a proof boundary
clearer. Belvedere is not a generic dashboard. It is a scenic overlook for a
defensive landscape, with enough structure that the scenery can be audited.
