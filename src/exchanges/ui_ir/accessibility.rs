//! Accessibility proof helpers for traceable UI widgets.

use elicit_ui::{
    AccessKitUiBackend, FocusDescriptor, KeyboardDescriptor, TargetDescriptor, WcagFocusFactory,
    WcagKeyboardFactory, WcagTargetFactory, WidgetId,
};
use tracing::instrument;

use super::UiIrError;

#[instrument(level = "debug", skip(backend, widget))]
pub(super) fn establish_interactive_proofs(
    backend: &AccessKitUiBackend,
    widget: WidgetId,
) -> Result<(), UiIrError> {
    backend
        .build_keyboard_accessible(KeyboardDescriptor {
            widget,
            tab_index: 0,
            shortcut: None,
        })
        .map_err(UiIrError::elicit_ui)?;
    backend
        .build_target_minimum(TargetDescriptor {
            widget,
            width_px: 44.0,
            height_px: 44.0,
            adjacent_spacing_px: 8.0,
        })
        .map_err(UiIrError::elicit_ui)?;
    backend
        .build_focus_visible(FocusDescriptor {
            widget,
            indicator_area_px: 88.0,
            indicator_contrast: 3.0,
        })
        .map_err(UiIrError::elicit_ui)?;
    Ok(())
}
