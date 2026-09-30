//! Amenable standards minted from prior-art `elicit_ui` props.

use amenable_derive::Standard;

/// Marker for standards minted from existing `elicit_ui` proposition types.
pub trait MintedElicitUiStandard {
    /// The prior-art `elicit_ui` proposition this standard names.
    type Prop;

    /// Stable `elicit_ui` proposition name.
    const PROP_NAME: &'static str;

    /// Why Belvedere needs this proposition at the UI IR boundary.
    const PURPOSE: &'static str;
}

/// Amenable standard minted from `elicit_ui::IrSourced`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::elicit_ui_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct ElicitUiIrSourcedStandard;

impl MintedElicitUiStandard for ElicitUiIrSourcedStandard {
    type Prop = elicit_ui::IrSourced;

    const PROP_NAME: &'static str = "elicit_ui::IrSourced";
    const PURPOSE: &'static str = "the UI tree was projected from Belvedere auditor evidence";
}

/// Amenable standard minted from `elicit_ui::WcagVerified`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::elicit_ui_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct ElicitUiWcagVerifiedStandard;

impl MintedElicitUiStandard for ElicitUiWcagVerifiedStandard {
    type Prop = elicit_ui::WcagVerified;

    const PROP_NAME: &'static str = "elicit_ui::WcagVerified";
    const PURPOSE: &'static str = "the emitted UI IR carries WCAG proof credentials";
}

/// Amenable standard minted from `elicit_ui::RenderComplete`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::elicit_ui_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct ElicitUiRenderCompleteStandard;

impl MintedElicitUiStandard for ElicitUiRenderCompleteStandard {
    type Prop = elicit_ui::RenderComplete;

    const PROP_NAME: &'static str = "elicit_ui::RenderComplete";
    const PURPOSE: &'static str = "a downstream renderer can close the projection custody chain";
}

/// Amenable standard minted from `elicit_ui::RolePreserved`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Standard)]
#[standard(
    basis = "Self",
    provenance = "crate::contracts::elicit_ui_provenance()",
    provenance_type = "amenable::RustLanguageProvenance"
)]
pub struct ElicitUiRolePreservedStandard;

impl MintedElicitUiStandard for ElicitUiRolePreservedStandard {
    type Prop = elicit_ui::RolePreserved;

    const PROP_NAME: &'static str = "elicit_ui::RolePreserved";
    const PURPOSE: &'static str = "frontend bridge exchanges must preserve AccessKit roles";
}
