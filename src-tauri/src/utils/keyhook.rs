/**
 * keyboard hook
 */

pub mod keyhook_windows;

pub fn set_keyboard_hook()
{
	keyhook_windows::set_keyboard_hook();
}
