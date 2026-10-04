//! Amenable standards for Belvedere pipeline claims.

use amenable_derive::Standard;
use tracing::instrument;

/// Named Belvedere pipeline claim that can be registered as an Amenable standard.
pub trait BelvederePipelineStandard {
    /// Stable claim identifier used in certificates and assessments.
    const CLAIM_ID: &'static str;

    /// Pipeline phase that owns this claim.
    const PHASE: &'static str;

    /// Auditor-facing meaning of the claim.
    const DESCRIPTION: &'static str;
}

/// Input artifact fidelity standard ID.
pub const INPUT_FIDELITY_STANDARD_ID: &str = "belvedere.standard.input_fidelity";
/// Schema honesty standard ID.
pub const SCHEMA_HONESTY_STANDARD_ID: &str = "belvedere.standard.schema_honesty";
/// Landscape construction standard ID.
pub const LANDSCAPE_CONSTRUCTION_STANDARD_ID: &str = "belvedere.standard.landscape_construction";
/// Gap soundness standard ID.
pub const GAP_SOUNDNESS_STANDARD_ID: &str = "belvedere.standard.gap_soundness";
/// View-model fidelity standard ID.
pub const VIEW_MODEL_FIDELITY_STANDARD_ID: &str = "belvedere.standard.view_model_fidelity";
/// UI projection fidelity standard ID.
pub const UI_PROJECTION_FIDELITY_STANDARD_ID: &str = "belvedere.standard.ui_projection_fidelity";
/// Accessibility baseline standard ID.
pub const ACCESSIBILITY_BASELINE_STANDARD_ID: &str = "belvedere.standard.accessibility_baseline";
/// Report reproducibility standard ID.
pub const REPORT_REPRODUCIBILITY_STANDARD_ID: &str = "belvedere.standard.report_reproducibility";

/// Provenance shared by Belvedere's pipeline standards.
#[instrument(level = "debug")]
#[must_use]
pub fn belvedere_pipeline_provenance() -> amenable::RustLanguageProvenance {
    amenable::RustLanguageProvenance::for_source("belvedere", "belvedere::contracts")
}

/// Standard that input records are parsed without dropping identity or provenance.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct InputFidelityStandard;

impl BelvederePipelineStandard for InputFidelityStandard {
    const CLAIM_ID: &'static str = INPUT_FIDELITY_STANDARD_ID;
    const PHASE: &'static str = "ingestion";
    const DESCRIPTION: &'static str =
        "registry and assessment artifacts retain source identities and provenance";
}

/// Standard that unsupported Amenable fields stay visible to auditors.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct SchemaHonestyStandard;

impl BelvederePipelineStandard for SchemaHonestyStandard {
    const CLAIM_ID: &'static str = SCHEMA_HONESTY_STANDARD_ID;
    const PHASE: &'static str = "ingestion";
    const DESCRIPTION: &'static str = "unknown or unsupported Amenable fields remain accountable";
}

/// Standard that landscape models preserve source traceability.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct LandscapeConstructionStandard;

impl BelvederePipelineStandard for LandscapeConstructionStandard {
    const CLAIM_ID: &'static str = LANDSCAPE_CONSTRUCTION_STANDARD_ID;
    const PHASE: &'static str = "landscape";
    const DESCRIPTION: &'static str =
        "claims, proofs, roots, assessments, and gaps remain traceable to collected records";
}

/// Standard that missing or incomplete proof evidence is not marked complete.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct GapSoundnessStandard;

impl BelvederePipelineStandard for GapSoundnessStandard {
    const CLAIM_ID: &'static str = GAP_SOUNDNESS_STANDARD_ID;
    const PHASE: &'static str = "landscape";
    const DESCRIPTION: &'static str =
        "Belvedere may over-report uncertainty but must not hide missing proof evidence";
}

/// Standard that auditor view models retain traceable visible items.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct ViewModelFidelityStandard;

impl BelvederePipelineStandard for ViewModelFidelityStandard {
    const CLAIM_ID: &'static str = VIEW_MODEL_FIDELITY_STANDARD_ID;
    const PHASE: &'static str = "view_model";
    const DESCRIPTION: &'static str =
        "auditor rows, counters, badges, and navigation targets retain source IDs";
}

/// Standard that UI IR projection preserves auditor semantics.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct UiProjectionFidelityStandard;

impl BelvederePipelineStandard for UiProjectionFidelityStandard {
    const CLAIM_ID: &'static str = UI_PROJECTION_FIDELITY_STANDARD_ID;
    const PHASE: &'static str = "ui_ir";
    const DESCRIPTION: &'static str =
        "UI IR projection retains semantic roles, labels, relationships, and trace IDs";
}

/// Standard that emitted UI IR satisfies the available accessibility checks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct AccessibilityBaselineStandard;

impl BelvederePipelineStandard for AccessibilityBaselineStandard {
    const CLAIM_ID: &'static str = ACCESSIBILITY_BASELINE_STANDARD_ID;
    const PHASE: &'static str = "ui_ir";
    const DESCRIPTION: &'static str =
        "projected UI IR satisfies configured accessibility and verified-tree checks";
}

/// Standard that run certificates carry deterministic input and stage evidence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::belvedere_pipeline_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct ReportReproducibilityStandard;

impl BelvederePipelineStandard for ReportReproducibilityStandard {
    const CLAIM_ID: &'static str = REPORT_REPRODUCIBILITY_STANDARD_ID;
    const PHASE: &'static str = "run";
    const DESCRIPTION: &'static str =
        "the same input snapshot yields stable report and certificate evidence";
}
