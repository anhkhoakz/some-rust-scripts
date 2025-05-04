// src/ui.rs

use rfd::MessageDialog;

/**
 * Displays the break reminder window.
 */
pub struct BreakWindow;

impl BreakWindow {
    pub async fn show(message: &str) {
        MessageDialog::new()
            .set_title("Stretchly-RS Break")
            .set_description(message)
            .set_buttons(rfd::MessageButtons::Ok)
            .show();
    }
}
