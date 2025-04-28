/**
 * keyboard hook for Windows
 */
use crate::utils::notify;
use crate::utils::setting;

use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc, Arc, Mutex, OnceLock,
    },
    thread,
    time::Duration,
};

use core::panic;

use log::{debug, error, info};

use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        Input::{
            Ime::{
                ImmGetContext, ImmGetConversionStatus, HIMC, IME_CMODE_FULLSHAPE, IME_CMODE_NATIVE,
                IME_CONVERSION_MODE, IME_SENTENCE_MODE,
            },
            KeyboardAndMouse::{VK_BACK, VK_DOWN, VK_LEFT, VK_RETURN, VK_RIGHT, VK_UP},
        },
        WindowsAndMessaging::{
            CallNextHookEx, GetForegroundWindow, GetMessageW, SetWindowsHookExW,
            UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN,
        },
    },
};

/// キーコード記録の構造体
#[derive(Clone, Copy)]
pub struct Key {
    code: u32,             // キーコード
    ime_composition: bool, // IME変換中だったかどうか
}

impl Key {
    pub fn new(code: u32, ime_composition: bool) -> Self {
        Self {
            code,
            ime_composition,
        }
    }
}

/// キーコードの履歴を管理する構造体
struct KeyHistory {
    max_history_size: AtomicUsize, // キーコードの履歴の最大サイズ
    misstype_cnt: AtomicUsize,     // ミスタイプの回数
    history: Mutex<VecDeque<Key>>, // キーコードの履歴（スレッドセーフ）
}

fn is_ime_composition() -> bool {
    //! 現在のIMEが変換中かどうかを bool で返す
    //! * return `true` - IME変換中(確定前), `false` - IME変換なし、または確定済み
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return false;
        }

        let himc: HIMC = ImmGetContext(hwnd);
        if himc.0 == std::ptr::null_mut() {
            return false;
        }

        let mut conversion: IME_CONVERSION_MODE =
            windows::Win32::UI::Input::Ime::IME_CONVERSION_MODE(0);
        let mut sentence: IME_SENTENCE_MODE = windows::Win32::UI::Input::Ime::IME_SENTENCE_MODE(0);

        if ImmGetConversionStatus(himc, Some(&mut conversion), Some(&mut sentence)).as_bool() {
            // IME_CMODE_NATIVE         : 日本語変換モード
            // IME_CMODE_FULLSHAPE      : 全角モード
            return (conversion & (IME_CMODE_NATIVE | IME_CMODE_FULLSHAPE))
                != windows::Win32::UI::Input::Ime::IME_CONVERSION_MODE(0);
        } else {
            return false;
        }
    }
}

impl KeyHistory {
    pub fn new(max_history_size: usize) -> Self {
        //! コンストラクタ
        Self {
            max_history_size: AtomicUsize::new(max_history_size),
            misstype_cnt: AtomicUsize::new(0),
            history: Mutex::new(VecDeque::with_capacity(max_history_size)),
        }
    }

    pub fn clear(&self) {
        //! 履歴を全削除する
        let mut history = self.history.lock().unwrap();
        history.clear();
        self.misstype_cnt.store(0, Ordering::Relaxed);
    }

    pub fn change_max_history_size(&self, max_history_size: usize) {
        //! 履歴のサイズを変更する.
        //! サイズが小さくなる場合、古い履歴から削除される.
        //! 削除されるキーがBackSpaceなら、misstype_cntを減らす

        self.max_history_size
            .store(max_history_size, Ordering::Relaxed);
        let mut history = self.history.lock().unwrap();
        while history.len() > max_history_size {
            let oldest = history.pop_front();
            if let Some(old) = oldest {
                if old.code == VK_BACK.0 as u32 && self.misstype_cnt.load(Ordering::Relaxed) > 0 {
                    self.misstype_cnt.fetch_sub(1, Ordering::Relaxed);
                }
            }
        }
    }

    fn add_key(&self, key: &Key) {
        //! 履歴にキーを追加する.
        //! max_history_sizeを超えた場合は、最も古いキーを自動で削除する.
        //! 削除するキーがBackSpaceなら、missype_cntを減らす.

        let mut history = self.history.lock().unwrap();
        let max_history_size = self.max_history_size.load(Ordering::Relaxed);

        while history.len() >= max_history_size {
            let oldest = history.pop_front();
            if let Some(old) = oldest {
                if old.code == VK_BACK.0 as u32 && self.misstype_cnt.load(Ordering::Relaxed) > 0 {
                    self.misstype_cnt.fetch_sub(1, Ordering::Relaxed);
                }
            }
        }
        history.push_back(key.clone());
    }

    pub fn regist_key(&self, input_key: &Key) {
        //! 履歴にキー入力を記録し、ミスタイプの計算を行う
        let (prev, prev_prev) = {
            let history = self.history.lock().unwrap();
            let len = history.len();
            (
                history.get(len.wrapping_sub(1)).copied(),
                history.get(len.wrapping_sub(0)).copied(),
            )
        };

        let vk_down = VK_DOWN.0 as u32;
        let vk_up = VK_UP.0 as u32;
        let vk_left = VK_LEFT.0 as u32;
        let vk_right = VK_RIGHT.0 as u32;
        let vk_back = VK_BACK.0 as u32;
        let vk_enter = VK_RETURN.0 as u32;

        if input_key.code == vk_back {
            match (prev, prev_prev) {
                (Some(prev1), Some(_prev2)) if prev1.code == vk_back => {
                    // 連続したBackSpaceはカウントしない、履歴に追加しない
                    debug!("Detect Continuous BackSpace");
                }
                (Some(prev1), Some(prev2)) if prev1.code == vk_enter && prev2.ime_composition => {
                    // 変換中 -> Enter(変換確定) -> BackSpace の時だけミスタイプ修正とする
                    debug!("Detect BackSpace after composition");
                    self.misstype_cnt.fetch_add(1, Ordering::Relaxed);
                    self.add_key(input_key);
                }
                (Some(prev1), Some(_prev2))
                    if prev1.code == vk_up
                        || prev1.code == vk_down
                        || prev1.code == vk_right
                        || prev1.code == vk_left =>
                {
                    debug!("Detect BackSpace after allow key");
                    if setting::MisstypeConfig::get_afterallow() {
                        // 矢印キーの後をミスタイプとする設定が有効の場合のみカウント
                        self.misstype_cnt.fetch_add(1, Ordering::Relaxed);
                        self.add_key(input_key);
                    }
                }
                _ => {
                    // 通常のBackSpaceはミスタイプの修正にカウント
                    debug!("Detect BackSpace");
                    self.misstype_cnt.fetch_add(1, Ordering::Relaxed);
                    self.add_key(input_key);
                }
            }
        } else {
            self.add_key(input_key);
        }
    }

    pub fn get_recent_mistype_cnt(&self) -> usize {
        //! 直近のミスタイプの回数を返す
        self.misstype_cnt.load(Ordering::Relaxed)
    }
}

// `HHOOK` を `Send` にするためのラッパー型
#[derive(Clone, Copy)]
struct SafeHHook(HHOOK);

unsafe impl Send for SafeHHook {}
unsafe impl Sync for SafeHHook {}

static HISTORY: OnceLock<KeyHistory> = OnceLock::new(); // KeyHistoryのimplがスレッドセーフとなっているので、排他処理は不要
static HOOK: OnceLock<Arc<Mutex<Option<SafeHHook>>>> = OnceLock::new();
static TX: OnceLock<mpsc::Sender<Key>> = OnceLock::new();

const NOTIFY_TITLE: &str = "OopsTime detected a lot of mistype!";
const NOTIFY_BODY: &str = "Shall we take a coffee break?";

fn regist_key(key: &Key) {
    //! グローバル変数 HISTORY のキー登録を行う関数
    let history = HISTORY.get().expect("HISTORY not initialised");

    history.regist_key(key);
}

fn get_recent_mistype_cnt() -> usize {
    //! グローバル変数 HISTORY のミスタイプ回数の取得を行う関数
    HISTORY
        .get()
        .expect("HISTORY not initialized")
        .get_recent_mistype_cnt()
}

unsafe extern "system" fn keyboard_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let kb_data: &KBDLLHOOKSTRUCT = &*(l_param.0 as *const KBDLLHOOKSTRUCT);

        if w_param == WPARAM(WM_KEYDOWN as usize) {
            let keycode = kb_data.vkCode as u32;
            let key = Key::new(keycode, is_ime_composition());

            if let Some(tx) = TX.get() {
                match tx.send(key) {
                    Ok(()) => debug!("Send key: {}", key.code),
                    Err(mpsc::SendError(e)) => error!("Failed to send key: {}", e.code),
                }
            }
        }
    }

    let hook_guard = HOOK.get().unwrap().lock().unwrap();
    if let Some(hook) = *hook_guard {
        return CallNextHookEx(Some(hook.0), n_code, w_param, l_param);
    }

    LRESULT(0)
}

fn set_keyboard_hook() {
    unsafe {
        // キーボードフック処理を登録
        let hook: HHOOK = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), None, 0)
            .unwrap_or_else(|_| {
                error!("Failed to set hook");
                panic!("Failed to set hook");
            });

        // `OnceLock` を初期化
        let _ = HOOK.set(Mutex::new(Some(SafeHHook(hook))).into());

        let mut msg = std::mem::zeroed();
        while GetMessageW(&mut msg, None, 0, 0).0 != 0 {
            debug!("Received message: {}", msg.message);
        }

        // フックを解除
        if let Some(hook) = HOOK.get().unwrap().lock().unwrap().take() {
            UnhookWindowsHookEx(hook.0).unwrap_or_else(|_| {
                error!("Failed to unhook");
                panic!("Failed to unhook");
            });
        }
    }
}

fn regist_key_daemon(rx: mpsc::Receiver<Key>) {
    //! keyhookから送信したキー情報を受信し、履歴に登録する
    loop {
        match rx.recv() {
            Ok(key) => {
                debug!("Recv key: {}", key.code);
                regist_key(&key);
            }
            Err(e) => {
                debug!("Recv error: {}", e);
                break; // Finish regist key deamon
            }
        }
    }
    info!("Exit regist key daemon");
}

fn make_time_str(sec: u64) -> String {
    //! 秒を %H:%M:%S に変換する
    let h = sec / 3600;
    let m = (sec % 3600) / 60;
    let s = sec % 60;
    format!("{}:{:02}:{:02}", h, m, s)
}

fn mistype_rate_monitor_daemon() {
    //! 一定時間ごとにミスタイプ率を取得し、ミスタイプ率が閾値を超えると通知を送信する
    loop {
        // 設定が変更された場合でも即座に反映されるように、loopの中で値を取得する
        let interval = setting::MisstypeConfig::get_interval();
        let threshold = setting::MisstypeConfig::get_threshold();
        let count = setting::MisstypeConfig::get_count();
        let thres_cnt = (threshold * count as f64).floor() as usize;
        let mistype_cnt = get_recent_mistype_cnt();
        debug!(
            "Current mistype count: {},  next monitoring is {} later...",
            mistype_cnt,
            make_time_str(interval)
        );

        if mistype_cnt > thres_cnt {
            match notify::send_notify(NOTIFY_TITLE, NOTIFY_BODY) {
                Ok(_) => info!("Notified high mistype rate detected!"),
                Err(err) => error!("Fail to send notify detecting hight mistype rate: {}", err),
            }
            /* 閾値を超えた状態のままにすると、ずっと通知が送信されるので、
             * 通知を送信した後は履歴を削除する */
            HISTORY.get().expect("HISTORY not initialized").clear();
        }

        thread::sleep(Duration::from_secs(interval));
    }
    info!("Exit mistype rate monitor daemon");
}

pub fn init_keyhook() {
    // グローバル変数 HOOK の初期化
    HOOK.set(Arc::new(Mutex::new(Some(SafeHHook(HHOOK(
        std::ptr::null_mut(),
    ))))))
    .unwrap_or_else(|_| {
        error!("Failed to initialize SafeHHOOK structure");
        panic!("Failed to initialize SafeHHOOK structure");
    });

    HISTORY
        .set(KeyHistory::new(setting::MisstypeConfig::get_count()))
        .unwrap_or_else(|_| {
            error!("Failed to initialize keyboard input history");
            panic!("Failed to initialize keyboard input history");
        });

    let (tx, rx) = mpsc::channel::<Key>();
    TX.set(tx).expect("TX already set");

    thread::spawn(move || {
        // 別スレッドでキー履歴を処理する
        debug!("run regist key daemon on {:?}", thread::current().id());
        regist_key_daemon(rx);
    });

    thread::spawn(|_| {
        // 別スレッドでキーフックの処理を行う
        debug!("run set keyboard hook on {:?}", thread::current().id());
        set_keyboard_hook();
    });

    thread::spawn(|_| {
        // 別スレッドミスタイプ率を監視
        debug!(
            "run mistype rate monitor daemon on {:?}",
            thread::current().id()
        );
        mistype_rate_monitor_daemon();
    });
}
