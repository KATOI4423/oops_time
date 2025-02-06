/**
 * keyboard hook for Windows
 */

use log::{
	error,
	info,
};
use simplelog::{
	ColorChoice, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger
};
use windows::Win32::{
	UI::WindowsAndMessaging::{
		CallNextHookEx,
		GetMessageW,
		HHOOK,
		KBDLLHOOKSTRUCT,
		SetWindowsHookExW,
		UnhookWindowsHookEx,
		WH_KEYBOARD_LL,
		WM_KEYDOWN,
	},
	Foundation::{
		LPARAM,
		LRESULT,
		WPARAM,
	},
	System::LibraryLoader::GetModuleHandleW,
};
use std::{
	fs::File,
	sync::{
		Mutex,
		OnceLock,
	},
};

pub fn init_logger()
{
	let log_file = File::create("oopstime.log").expect("Failed to create log file.");
	WriteLogger::init(LevelFilter::Info, Config::default(), log_file).expect("Failed to initialize logger");
	info!("Logger initialized successfully");
}


// `HHOOK` を `Send` にするためのラッパー型
#[derive(Clone, Copy)]
struct SafeHHook(HHOOK);

unsafe impl Send for SafeHHook {}
unsafe impl Sync for SafeHHook {}

// スレッドセーフな `OnceLock` を使用
static HOOK: OnceLock<Mutex<Option<SafeHHook>>> = OnceLock::new();

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT
{
	if n_code >= 0 {
		let kb_data: &KBDLLHOOKSTRUCT = &*(l_param.0 as *const KBDLLHOOKSTRUCT);

		if w_param == WPARAM(WM_KEYDOWN as usize) {
			println!("Key pressed: {}", kb_data.vkCode);
			info!("Key pressed: {}", kb_data.vkCode);
		}
	}

	let hook_guard = HOOK.get().unwrap().lock().unwrap();
	if let Some(hook) = *hook_guard {
		return CallNextHookEx(Some(hook.0), n_code, w_param, l_param);
	}

	LRESULT(0)
}

pub fn set_keyboard_hook()
{
	unsafe {
		let module_handle = GetModuleHandleW(None).expect("Failed to get module handle");

		let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), Some(module_handle.into()), 0)
			.expect("Failed to set hook");

		// `OnceLock` を初期化
		let mutex = Mutex::new(Some(SafeHHook(hook)));
		let _ = HOOK.set(mutex);

		let mut msg = std::mem::zeroed();
		while GetMessageW(&mut msg, None, 0, 0).0 != 0 {
			println!("Received message: {}", msg.message);
		}

		// フックを解除
		if let Some(hook) = HOOK.get().unwrap().lock().unwrap().take() {
			UnhookWindowsHookEx(hook.0).expect("Failed to unhook");
		}
	}
}
