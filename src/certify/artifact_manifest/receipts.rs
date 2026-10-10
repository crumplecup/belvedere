//! Receipt bundles used while building artifact manifests.

use crate::certify::{
    IngestionReceipt, LandscapeDiffReceipt, LandscapeDiffReviewReceipt, LandscapeReceipt,
    UiIrReceipt, ViewModelReceipt,
};

use derive_getters::Getters;

/// Receipt evidence for one projected Belvedere snapshot in an artifact manifest.
#[derive(Debug, Clone, Copy, Getters)]
pub(crate) struct BelvedereSnapshotManifestReceipts<'receipt> {
    #[getter(copy)]
    registry: &'receipt IngestionReceipt,
    #[getter(copy)]
    assessment: &'receipt IngestionReceipt,
    #[getter(copy)]
    landscape: &'receipt LandscapeReceipt,
    #[getter(copy)]
    view_model: &'receipt ViewModelReceipt,
    #[getter(copy)]
    ui_ir: &'receipt UiIrReceipt,
}

impl<'receipt> BelvedereSnapshotManifestReceipts<'receipt> {
    /// Bundle the receipt evidence for one projected snapshot.
    #[must_use]
    pub(crate) const fn new(
        registry: &'receipt IngestionReceipt,
        assessment: &'receipt IngestionReceipt,
        landscape: &'receipt LandscapeReceipt,
        view_model: &'receipt ViewModelReceipt,
        ui_ir: &'receipt UiIrReceipt,
    ) -> Self {
        Self {
            registry,
            assessment,
            landscape,
            view_model,
            ui_ir,
        }
    }
}

/// Receipt evidence for a landscape diff run artifact manifest.
#[derive(Debug, Clone, Copy, Getters)]
pub(crate) struct BelvedereDiffManifestReceipts<'receipt> {
    previous: BelvedereSnapshotManifestReceipts<'receipt>,
    current: BelvedereSnapshotManifestReceipts<'receipt>,
    #[getter(copy)]
    diff: &'receipt LandscapeDiffReceipt,
    #[getter(copy)]
    diff_review: &'receipt LandscapeDiffReviewReceipt,
    #[getter(copy)]
    diff_review_ui_ir: &'receipt UiIrReceipt,
}

impl<'receipt> BelvedereDiffManifestReceipts<'receipt> {
    /// Bundle the receipt evidence for a two-snapshot landscape diff.
    #[must_use]
    pub(crate) const fn new(
        previous: BelvedereSnapshotManifestReceipts<'receipt>,
        current: BelvedereSnapshotManifestReceipts<'receipt>,
        diff: &'receipt LandscapeDiffReceipt,
        diff_review: &'receipt LandscapeDiffReviewReceipt,
        diff_review_ui_ir: &'receipt UiIrReceipt,
    ) -> Self {
        Self {
            previous,
            current,
            diff,
            diff_review,
            diff_review_ui_ir,
        }
    }
}
