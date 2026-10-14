# Belvedere Completion Plan

Status: active. This is the implementation plan for the remaining work in the
[strategic plan](BELVEDERE_STRATEGIC_PLAN.md). It targets a reviewable
`elicit_ui` IR, not a renderer. The remaining work is a sequence of contract
amendments at named exchange boundaries. Implementations follow those amended
interfaces; a stage is not complete merely because it emits a receipt.

## Evidence Map

The following is the source contract **today**, not a proposed schema. Amenable
builds `dump-registry` from `inventory` in
`../amenable/crates/amenable/src/registry_dump/collect.rs`. Its output has no
top-level version field and contains these five arrays:

- `evidence_links` has `name`, `basis`, and fan-out `index`. These are static
  claim-to-basis registrations. A root registers a self-link. Belvedere's
  `LandscapeClaim::is_root` also treats any node with no bases as a root;
  distinguish an explicit self-link from absent topology.
- `proof_records` has `evidence` and `verifier`. It registers a backend proof
  for an evidence type. The dump omits `describe()` and does not run a
  verifier. Belvedere's `LandscapeClaim::proofs`, `BackendCoverageItem.covered`,
  and chain `complete` currently read as verification coverage; these need
  registration-specific names and labels.
- `contract_records` has `evidence`, `verifier`, `kind`, and `fragment`. This
  is registered native requires/ensures text. Its presence is not an executed
  check; preserve the fragment and source identity.
- `witness_export_records` has verifier, evidence, destination, support
  counts, and recursive `artifact`. Counts such as `checked` and `trusted`
  classify support, not verifier execution. Belvedere stores `artifact` as
  JSON in `WitnessExportRecord`, but `WitnessSurfaceSummary::from_record`
  drops it and downstream views project only counts. The record survives,
  but its payload granularity does not; expose the tree without upgrading
  its status.
- `UnsupportedField` currently carries only record and field names. The
  decoded `extra` map holds JSON values, but the ingestion receipt omits
  them. A later exchange cannot claim it preserved unknown payload detail
  by counting or naming those fields. Preserve inspectable values and paths.
- `kani_proofs` has `id`, `harness`, and `package`: harness registrations only.
  There is no status, scope, or failure reason. `KaniProofRecord` currently
  defaults missing status to `Verified`, and the main fixture supplies invented
  execution fields. Remove execution fields from this source model.

The assessment producer is
`../amenable/crates/amenable/src/assessment/record.rs`. It writes JSON Lines
with `version: "0.1.0"`, `assessment_id`, `proof_id`, reviewer, timestamp,
six rubric scores, recommendation, resolution path, and comment. It also
accepts legacy `schema_version: 1` and `timestamp_unix_seconds`. Amenable
rejects missing or conflicting version/timestamp fields, blank IDs/reviewer/
comment, current records without assessment ID or resolution path, incompatible
recommendation/path pairs, and rubric scores outside 0..=4. Belvedere's
`AssessmentStreamIngestion` currently only deserializes; `AssessmentRecord`
turns a missing timestamp into zero. This is a schema-fidelity defect.

Amenable already writes a separate Kani result ledger in
`../amenable/crates/amenable/src/kani.rs`: CSV header
`proof_id,timestamp,status`, one latest row per proof ID, with statuses
`passed`, `failed`, or `timeout`. `amenable verify kani --results <path>`
produces it. The ledger lacks harness/package, source revision, verifier
version, scope, and diagnostic reason. It can establish a *recorded result*
for an exact registered ID, but cannot establish that the current source tree
was verified. There is no equivalent structured result artifact for Creusot
or Verus in the inspected `amenable` CLI. Do not silently infer one from
`ProofRecord`.

Certificate reality is similarly specific. `amenable_core::Certificate`
provides an ID, subject, and ordered frozen `MetadataEntry { key, value }`
iterator. `amenable_std::CertRegistry` issues sequential `CertId(u64)` values
local to a registry instance. `issue_standard_certificate` delegates to
`issue_provenance_certificate`, so the certificate does not carry a standard
or claim ID. The only current file writer,
`amenable_std/src/rust_std/certificate.rs`, writes `Display` text to
`*.provenance.txt`; `dump-registry` contains no certificates. Belvedere's
`MetadataUiIrInput::from_certificate` is an in-process projection, not an
artifact ingestion path. A structured export and explicit root binding must
be produced; parsing the text would invent a contract Amenable has not made.

One more producer constraint affects every real-repository test:
`amenable dump-registry` runs in the standalone Amenable binary and walks only
registrations linked into **that binary**. Its `RegistryDump::collect` is
private. It cannot inspect arbitrary downstream code by pointing it at a
repository. A target-linked executable or test must call a public Amenable
export API to write that target's registrations. This is a required producer
change, not a validation chore to leave for later.

## Decisions For The Remaining Work

- A registration, a recorded verifier result, a source-bound verified result,
  a human assessment, and an issued certificate are distinct evidence kinds.
  The current inputs support the first, second, fourth, and an in-process form
  of the fifth. They do **not** support a source-bound verification claim.
- Keep the existing five-array registry shape readable. Its lack of a version
  marker is part of the accepted legacy format; a new producer envelope must
  be versioned and must not silently reinterpret old fields.
- Use occurrence-level source references: artifact digest plus array name and
  index, or JSONL/CSV line. Semantic IDs (`proof_id`, evidence name, verifier)
  are join keys, not globally unique source-record IDs. Duplicate semantic
  keys remain visible and are never resolved by map overwrite.
- No implicit certificate-to-claim join on display subject, local numeric
  `CertId`, matching text, or metadata key. A producer-supplied root evidence
  name is required for a grounded join; otherwise display the certificate as
  unbound. The local ID is namespaced by its certificate bundle identity.
- The existing chain traversal is a *Belvedere-derived registration graph*.
  Amenable's `proof_chain_for_verifiers` also reconstructs registered proofs,
  not executed ones, and is available only in-process. Keep this distinction
  in the view model and IR; do not call either an execution certificate.
- Belvedere's stage/run certificates attest to its transformations and bound
  inputs. They do not certify the source repository's proof outcomes.

## Exchange Law

Fidelity, completeness, and granularity are not later audits of the
implementation. They are postconditions of each exchange over the Amenable
artifacts Belvedere actually receives. Belvedere is an umpire of those
artifacts, not a prover of the source code. For each accepted source
occurrence, an exchange must identify what it preserved, what it derived,
what it could not interpret, and where a reviewer can find that disposition.
Aggregation is allowed, but an aggregate must retain its members' occurrence
references and a path back to their full payloads.
An unrecognized relationship is an explicit unbound or unknown result, never
an invented source link. A failed required precondition rejects the exchange;
an optional artifact that was not supplied is a distinct input state.

Evidence strength is monotone only when new evidence is attached. A registry
proof registration cannot become a successful execution, a CSV pass cannot
become verification of the current source revision, a root cannot arise from
an absent basis edge, and a certificate cannot bind to a claim by matching
display text. These are contract violations, even if the UI looks plausible.

The current interface does not enforce this law yet. `BelvedereContract`
names `Input`, `Output`, and `Receipt`, but its `requires()` and `ensures()`
return descriptive strings. `BelvedereExchange::exchange` returns an output
and receipt, while stage certificates record boolean invariant checks and
their digest without requiring every check to pass. Some existing checks
compare counts or ID lists derived from the same data, not the Amenable input
against the transformed output. Such checks can pass after a dropped record
or invented association. A certificate containing `passed: false` is evidence
of a failed check, not a successful attestation. Amenable's own
`Requires<V>`/`Ensures<V>` have typed inputs and backend-specific bounds;
Belvedere's string clauses are not substitutes for those predicates or for
runtime assertions.

Before adding sources, amend the interface so each boundary has typed source
occurrences, explicit dispositions, and executable pre/postcondition checks.
Keep the readable `Requires`/`Ensures` clauses, but associate each critical
clause with a named check over actual input, output, and receipt evidence.
Every emitted stage certificate must identify those checks and their
evidence. Each exchange must run its required checks before returning a
successful output/receipt pair; the success path must refuse to issue a
certificate when a required check fails. Report rejected input and failed
postconditions through the normal error surface. The compiler can force
updated types through consumers; it cannot prove semantic conservation by
itself, so counterexample tests are part of each exchange's contract.

The shared accounting shape should stay small: artifact identity, a structured
occurrence and field location, semantic join keys kept separately, and a
disposition such as represented, explicitly unbound, or unsupported. Not every
stage needs the same output item type. A transition check must compare the
actual input occurrences and payload fields against output references, not
two counters populated from the same list. For a derived fact, record its
derivation rule and all supporting occurrences. A check must also rule out
unsupported status promotion.

### Required Contract Types And Assertions

Each exchange needs four typed roles, even if their concrete Rust names vary
by stage:

1. **Input evidence:** immutable accepted artifact or prior-stage output,
   including occurrence identity, field paths, original payload, and
   provenance. Optional absence is modeled explicitly, not as an empty
   artifact.
2. **Output evidence:** semantic facts with stable identities, their support
   references, and any declared normalization or derivation rule.
3. **Accounting evidence:** a relation from each input occurrence and payload
   field to one or more output identities/field paths, or to a permitted
   unbound/unsupported disposition. The disposition vocabulary is specific
   to the input kind; a known required field cannot be marked unsupported
   merely to make the accounting total.
4. **Assertion evidence:** named, versioned results computed from the actual
   input, output, and accounting relation; each result records what it checked
   and what counterexample caused failure. The receipt binds this evidence
   and the input/output digests, but is not itself proof that a predicate ran.

For every transition, assert both directions of the relation:

- **Completeness:** every accepted input occurrence has a permitted,
  reviewable disposition. Where the contract requires projection, it reaches
  an output fact or a navigable gap; a warning or count is not a substitute.
- **Fidelity:** every output assertion cites real input occurrences or a
  declared derivation, and its fields/status match those sources under the
  stated transformation rule. An unexplained output has no authority.
- **Granularity:** every input payload field, nested value, and meaningful
  ordering/duplication remains addressable at its original specificity after
  transformation. A summary may omit detail on its face, but the reviewer
  must be able to reach the exact detail through the output model and IR.
  Unsupported fields need inspectable values, not only names or counts.
- **Strength preservation:** outputs never imply execution, provenance,
  certificate binding, or source-currentness beyond the input evidence.
- **Binding:** the checked input, output, accounting relation, and receipt
  are the same objects whose digests enter the stage/run certificates.

Aggregation may be many-to-one and expansion one-to-many; neither weakens
these obligations. A claim summary must identify all contributing source
occurrences, and each generated field must cite its origin or derivation.
The immutable raw artifact remains digest-bound for exact byte inspection;
that fallback alone does not satisfy granularity if Belvedere's navigation
cannot reach the corresponding field or nested value. Normalization must
retain the original value and state its rule. The contract tests field-level
reachability, not just record-level presence.
Tests should mutate one side at a time (drop an input, invent an output,
alter or delete a nested field, reorder repeated values, change a join key)
so a check cannot pass merely because two counts or ID lists were built from
the same mistaken projection.

This obligation applies to every existing contract, including registry and
assessment ingestion, landscape modeling, auditor navigation, ordinary and
metadata `elicit_ui` projection, landscape diff, diff review, and diff-review
IR projection. New Kani-result and certificate-bundle ingestion contracts
enter under the same rule. No parallel metadata or diff path is exempt.

## Boundary Obligations

**Raw artifact to decoded evidence:** assert accepted schema, exact field
meaning, one disposition per input occurrence, and retained locator/digest.
Reject malformed required data; preserve allowed unknown fields as
unsupported. Check the producer's record positions against decoded record
positions, and compare every required field and nested payload value under
the declared normalization. Preserve original bytes for exact inspection.
Reject duplicate JSON object keys unless the accepted format explicitly
permits them and the decoder represents each occurrence; a map overwrite
cannot satisfy granularity.

**Decoded evidence to landscape:** assert that every occurrence is
represented, deliberately unbound, or unsupported. Joins use declared keys;
derived roots and statuses cite their rules and sources. Preserve orphans and
unknown basis as gaps; never fabricate a source or verified result. Check the
graph's edges, registrations, assessments, and certificate bindings against
their specific input families, not only a combined total. In particular,
preserve the recursive witness artifact alongside its summary counts.

**Landscape to view model:** assert that every displayed count, status, gap,
field, and action target resolves to graph evidence or an explicit derivation.
Expose absent sources and unbound items; omit unsupported assertions, not
their evidence. Check that each graph fact required for review is reachable
from navigation, including its detailed payload. A summary-only path with no
way to inspect the contributing fields fails granularity.

**View model to `elicit_ui`:** assert preservation of semantic content, order,
source references, and target resolution. Interactive roles need resolvers;
otherwise emit noninteractive content. Check IR nodes against view-model
items and fields, not against the projection's own widget counts. Lost
evidence fails the exchange. A source-field locator needs a resolvable IR
target, not merely a raw artifact digest in a receipt.

**Receipts to certificates and manifest:** assert end-to-end binding of input
digests, optional-source state, dispositions, outputs, and named passing
checks. A failed required check prevents a success certificate; no source
may disappear from the manifest.

**Previous/current snapshots to diff:** assert that both sides use the same
snapshot contract and each reported change identifies before/after evidence.
Check additions, removals, changes, and unchanged identities against both
source snapshots. Unmatched or unsupported evidence stays visible, not an
inferred deletion.

## Delivery Order

### 1. Make Exchange Assertions Real

**Interface amendment:** define the checkable obligations and failure semantics
on `BelvedereContract`/`BelvedereExchange` and their receipt/certificate path.
Put input, output, accounting, and assertion evidence into contract types
before changing parsers or graphs. Define stage-specific permitted
dispositions and field-level predicates for fidelity and granularity. Decide
per check whether it is a precondition, a required postcondition, or a
diagnostic that may remain false without attestation.

**Compiler follow-through:** update `src/contracts/contract.rs`,
`src/exchanges/exchange.rs`, `src/certify/invariant/`,
`src/certify/stage_certificate.rs`, and the run certificate/manifest builders.
Replace count-only claims of completeness where necessary with comparisons
against independent source and output evidence. A failed required check must
not yield a successful stage/run certificate.

**Counterexamples:** two distinct source occurrences with identical content,
a dropped output reference hidden by equal counts, an invented source link,
an altered value with a preserved ID, an invented output claim, and a false
required invariant must each fail the appropriate exchange or produce an
explicit unresolved disposition. Dropping a witness-tree child while keeping
the witness record and counts must fail granularity. Existing good fixtures
still produce passing certificates.

### 2. Repair Ingestion And Attribution Contracts

**Interface amendment:** `RegistryDumpIngestionContract` accepts Amenable's
five-array dump as actually produced; `kani_proofs` has only `id`, `harness`,
and `package`. Nonstandard `status`, `scope`, and `failure_reason` are
unsupported raw fields, never execution evidence. The
`AssessmentStreamIngestionContract` accepts exactly the producer's current
and legacy shapes and rejects the malformed combinations in the evidence map.
Its successful output preserves timestamp spelling, source line, and each
assessment occurrence. Neither contract may manufacture an absent field.
Unknown fields keep their values and nested paths in typed unsupported-field
evidence, not just names in `UnsupportedField` or counts in a receipt.

**Compiler follow-through:** revise `src/contracts/ingestion/{registry,assessment}.rs`
and `src/exchanges/ingestion.rs`; replace content-derived IDs in
`DecodedRegistrySnapshot::source_record_ids` and assessment source IDs with
artifact digest plus structured array/index or line location. Use those
locators in `src/exchanges/view_model/sources.rs`, without parsing delimiters
inside IDs. Remove `src/exchanges/source_records.rs`'s all-record fallback.

**Counterexamples:** actual producer dump, unsupported extension fields,
every rejected assessment rule, duplicate identical records, punctuation in
names, duplicate JSON object keys, and a missing timestamp. Each accepted
input occurrence resolves to one locator, and unsupported values remain
inspectable. No fixture may still assert invented Kani execution status.

### 3. Specify Registration, Result, And Root Exchanges

**Interface amendment:** `LandscapeModelingContract` maps each registry
record to a typed registration fact or explicit unbound/gap disposition.
An explicit self-link alone establishes a *registered* root, not certified
provenance. A `ProofRecord`'s `(evidence, verifier)` is not a Kani `proof_id`;
assessments attach to an exact known Kani registration ID or remain unbound.
Chain and backend completeness mean
**registration completeness**, not successful verification.

Add a Kani-results ingestion contract for the existing CSV header and three
fields. Every row retains its line and artifact digest; a row joins only by
exact `kani_proofs.id` and has a represented, duplicate, or unmatched
disposition. A `passed`, `failed`, or `timeout` row is a recorded result whose
source revision is unknown. Absent optional ledger means `not supplied`.
No `verified current source` state exists until an Amenable producer emits
proof ID, package, harness, tool version, source revision/digest, timestamp,
status, and configuration, and the exchange checks the revision match.

**Compiler follow-through:** amend `LandscapeClaim`, graph/gap contracts,
backend and chain view items, `ArtifactKind`, receipt counts, JSON/CLI/IR
labels, and `src/exchanges/landscape.rs`. Preserve orphan assessments,
missing bases, unmatched CSV rows, and duplicate IDs as inspectable facts.
The CSV digest must reach landscape and run receipts.

**Counterexamples:** registry-only run, all three real CSV statuses, duplicate
and unmatched IDs, absent ledger, absent self-link, and orphan assessment.
None may silently turn into verified coverage or certified provenance.

### 4. Establish Producer-Side Exchange Contracts

**Target-linked registry:** Amenable must expose its existing owned registry
snapshot or a writer callable from a binary linked to the audited target.
The producer contract names the linked-program `inventory` scope and reuses
the five-array serializer. Its test target registers an `EvidenceLink`,
`ProofRecord`, contract, and Kani harness; its dump must contain those
occurrences, while a separate Amenable binary dump must not be mistaken for
the target. Record producer command and version. Do not fork the walk in
Belvedere. Until this interface exists, real external-repository validation
is blocked, not passed by synthetic fixtures.

**Certificate interchange:** at Amenable's issuing boundary, define a
versioned JSON bundle with bundle/issuer scope, local ID, subject, ordered
`{ key, value }` entries, and optional explicit root evidence name supplied
at standard certification. Ordinary provenance certificates have no implied
root. The writer serializes `CertRegistry::issued()` and
`Certificate::entries()` without sorting or deduplicating. Bundle identity
namespaces local `CertId`; the file digest identifies the artifact snapshot.
Text `.provenance.txt` output is not an interchange contract.

**Belvedere ingestion contract:** accept only known bundle versions; reject
invalid required fields and duplicate local IDs within one bundle. Preserve
field order, duplicate keys by ordinal, unknown optional fields in the
receipt, and unbound certificates. An absent optional bundle is recorded as
`not supplied`. Add the typed bundle, artifact kind, format root, and exchange
only after the producer contract is specified.

**Counterexamples:** real produced bundle round-trips ID, subject, explicit
root, and ordered duplicate fields; two bundles each containing `CertId(1)`
remain distinct; missing root stays unbound; duplicate IDs in one bundle
fail. Record the command and schema version with fixtures.

### 5. Thread The Contracts Through Normal And Diff Runs

**Interface amendment:** `BelvedereRunInput` and `LandscapeInput` carry
required sources and explicitly optional Kani results/certificate bundles.
Every accepted artifact receives a receipt, every absent option has a named
absence state, and every receipt digest reaches the landscape, stage/run
certificates, and manifest. Previous/current snapshots satisfy the same
contract before diffing. Invalid supplied files fail, never become absence.

Certificate facts in `LandscapeGraph` preserve bundle identity, local ID,
subject, explicit root, ordered fields, and source locator. A claim join
requires an exact explicit root name; unmatched or ambiguous joins are gaps.
Diff obligations account for additions, removals, binding changes, and
ordered field changes without inferring identity from subject text.

**Compiler follow-through:** extend `src/run.rs` and `src/cli/command.rs`
with optional `--kani-results <csv>` and `--certificates <json>` for normal
runs and previous/current forms for diff. Keep the two-input library
constructor and required positional inputs. Use one snapshot builder for
normal/diff, and update `src/certify/artifact_manifest/`, stage and run
invariants, graph, diff, and view model along the changed contract types.
Feed `MetadataUiIrInput` through this ordinary path, not a parallel path.

**Counterexamples:** normal and diff runs over real produced artifacts trace
every certificate field and CSV row to its occurrence; absent options, orphan
bindings, duplicate metadata keys, and two `CertId(1)` values remain
distinguishable. The old two-input path explicitly reports missing sources.

### 6. Assert Reviewer Semantics Through `elicit_ui`

**Interface amendment:** `AuditorNavigationContract` guarantees structured
overview, claim detail, registered chain, assessment history/work, witness
tree, certificates, gaps, and source references. Every visible assertion has
a source occurrence or named derivation rule; every detailed source field is
reachable through navigation. `ElicitUiProjectionContract` preserves those
items, nested fields, order, status distinctions, and source references in
IR. Interactive nodes carry a renderer-independent command and stable target
ID with a pure resolver; otherwise they are noninteractive evidence.

**Compiler follow-through:** amend `src/contracts/view_model/` and
`src/contracts/ui_ir/` first, then their exchanges. Replace long evidence
sentences in `src/exchanges/ui_ir/widgets.rs` button/link roles with
headings, groups, lists/tables, fields, status text, and actual actions.
Retain the existing `elicit_ui` verified-tree and WCAG proof path at desktop,
tablet, and mobile sizes; those proofs establish their named UI obligations,
not auditor task success or visual quality.

**Counterexamples:** a displayed fact with no source/derivation, a dropped
certificate field, a status upgraded in projection, an interactive role with
no resolver, or two targets for one command must fail the exchange. All
unsupported and unknown evidence remains inspectable.

### 7. Attest The Complete Product With Real Evidence

Use the target-linked producer for a real external Amenable-using crate and
for Belvedere through the **same** export procedure. Include a real
certificate bundle and Kani ledger where available; an empty assessment
stream is valid. Run normal and diff paths on both corpora and check each
boundary's disposition ledger, no status upgrades, all receipt/manifest
links, passing required invariants, and reviewer IR obligations. Keep
reproducible commands and sanitized fixtures. Do not fabricate review events
or verifier passes.

Belvedere's own claim/evidence ledger must distinguish standard/contract
roots, executable exchange checks, actual backend results, and human
assessments. Missing evidence stays unknown. A receipt or passing test is not
a Kani/Creusot/Verus proof. Review the IR with certification auditors for
hierarchy, uncertainty language, traceability, and task flow; renderer-only
visual judgments remain outside this plan. Update README and strategic-plan
claims to match only demonstrated capabilities.

For a code increment run focused Cargo tests/checks and the relevant Cordial
gate before committing. For planning-document-only edits run
`markdownlint-cli2`, not code gates. The plan is complete only when the
producer interfaces, normal/diff exchanges, reviewer IR, and evidence ledger
have passed their named obligations. Rendering remains a separate project.
