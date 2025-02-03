/**
 * main process
 */

mod keyhook;

fn main() {
	keyhook::set_keyboard_hook();
}
