///
/// Load License.html
///
use std::env;

include!(concat!(env!("OUT_DIR"), "/license.rs"));

#[tauri::command]
pub fn get_license_html() -> String {
    LICENSE_HTML.to_string()
}
