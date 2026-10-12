//! Stable CLI help text.

/// Stable CLI help text.
pub const BELVEDERE_CLI_HELP: &str = "\
usage: belvedere [--help] [<artifact> (--stdio | --file <path>)] <registry-dump.json> <proof-assessments.jsonl>
       belvedere diff [<artifact> (--stdio | --file <path>)] <previous-registry-dump.json> <previous-proof-assessments.jsonl> <current-registry-dump.json> <current-proof-assessments.jsonl>

Options:
  -h, --help                   Show this help text.
      --stdio                  Write the selected artifact as JSON to stdout.
      --file <path>            Write the selected artifact as JSON to a file.

Run artifacts:
      --certificate            Complete run certificate.
      --manifest               Run artifact manifest.
      --registry-receipt       Registry ingestion receipt.
      --assessment-receipt     Assessment ingestion receipt.
      --landscape              Landscape graph.
      --landscape-receipt      Landscape receipt.
      --view-model             Auditor navigation view model.
      --backend-coverage       Backend coverage items.
      --claim-explorer         Claim explorer items.
      --claim-details          Claim detail deep-dive items.
      --filter-facets          Auditor filter facets.
      --source-navigation      Source navigation map.
      --proof-chains           Proof-chain deep-dive items.
      --review-workflow        Auditor review workflow queue.
      --assessment-history     Assessment history items.
      --assessment-queue       Assessment queue items.
      --assessment-work        Assessment work items.
      --witness-surface        Witness surface items.
      --provenance-roots       Provenance root items.
      --gaps                   Conservative gap items.
      --view-model-receipt     View-model receipt.
      --ui-ir                  UI IR projection.
      --ui-ir-receipt          UI IR receipt.

Diff artifacts:
      --certificate            Landscape diff stage certificate.
      --run-certificate        Complete landscape diff run certificate.
      --manifest               Landscape diff run artifact manifest.
      --diff                   Certified landscape diff.
      --diff-receipt           Landscape diff receipt.
      --previous-registry-receipt
      --previous-assessment-receipt
      --previous-landscape
      --previous-landscape-receipt
      --previous-view-model
      --previous-backend-coverage
      --previous-claim-explorer
      --previous-claim-details
      --previous-filter-facets
      --previous-source-navigation
      --previous-proof-chains
      --previous-review-workflow
      --previous-assessment-history
      --previous-assessment-queue
      --previous-assessment-work
      --previous-witness-surface
      --previous-provenance-roots
      --previous-gaps
      --previous-view-model-receipt
      --previous-ui-ir
      --previous-ui-ir-receipt
      --current-registry-receipt
      --current-assessment-receipt
      --current-landscape
      --current-landscape-receipt
      --current-view-model
      --current-backend-coverage
      --current-claim-explorer
      --current-claim-details
      --current-filter-facets
      --current-source-navigation
      --current-proof-chains
      --current-review-workflow
      --current-assessment-history
      --current-assessment-queue
      --current-assessment-work
      --current-witness-surface
      --current-provenance-roots
      --current-gaps
      --current-view-model-receipt
      --current-ui-ir
      --current-ui-ir-receipt
";
