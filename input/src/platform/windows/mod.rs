//! Windows low-level keyboard hook and message loop.

mod keycodes;

use std::cell::RefCell;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Instant;

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::Threading::GetCurrentThreadId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetMessageW, PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_QUIT,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::{Error, Event, EventKind, RawCode};

thread_local! {
    // Windows calls the hook on the thread which installed it, so the sender
    // can be stored for that thread alone.
    static CONTEXT: RefCell<Option<CallbackContext>> = const { RefCell::new(None) };
}

struct CallbackContext {
    sender: Sender<Event>,
}

#[derive(Debug)]
pub(crate) struct ShutdownGuard {
    thread_id: Arc<AtomicU32>,
    worker: Option<JoinHandle<()>>,
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        let thread_id = self.thread_id.load(Ordering::Acquire);
        if thread_id != 0 {
            // SAFETY: posting WM_QUIT to the listener's message queue is the
            // documented cross-thread shutdown mechanism.
            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
            }
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

pub(crate) fn start(sender: Sender<Event>) -> Result<ShutdownGuard, Error> {
    let thread_id = Arc::new(AtomicU32::new(0));
    let worker_thread_id = Arc::clone(&thread_id);
    // Wait for the worker to install the hook before reporting success.
    let (ready_sender, ready_receiver) = std::sync::mpsc::sync_channel(1);

    let worker = thread::Builder::new()
        .name("legion-input-windows".into())
        .spawn(move || {
            // SAFETY: GetCurrentThreadId has no preconditions.
            let current_thread_id = unsafe { GetCurrentThreadId() };
            worker_thread_id.store(current_thread_id, Ordering::Release);

            // SAFETY: the callback has the required ABI and remains valid for
            // the process lifetime. A null module with thread id 0 is valid
            // for a low-level hook implemented in this process.
            let hook = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(callback), std::ptr::null_mut(), 0) };
            if hook.is_null() {
                let _ = ready_sender.send(Err(Error::Platform("SetWindowsHookExW returned null".into())));
                return;
            }

            CONTEXT.with(|context| {
                *context.borrow_mut() = Some(CallbackContext { sender });
            });
            let _ = ready_sender.send(Ok(()));

            // SAFETY: MSG is valid when zero-initialized and GetMessageW owns
            // its initialization on each iteration.
            let mut message: MSG = unsafe { std::mem::zeroed() };
            loop {
                // SAFETY: message points to writable storage for the duration
                // of the call; null HWND listens to this thread's queue.
                let result = unsafe { GetMessageW(&mut message, std::ptr::null_mut(), 0, 0) };
                if result <= 0 {
                    break;
                }
            }

            // SAFETY: hook was returned by SetWindowsHookExW in this thread
            // and has not previously been unhooked.
            unsafe {
                UnhookWindowsHookEx(hook);
            }
            CONTEXT.with(|context| {
                context.borrow_mut().take();
            });
        })
        .map_err(|error| Error::Platform(format!("could not spawn hook worker: {error}")))?;

    match ready_receiver.recv() {
        Ok(Ok(())) => Ok(ShutdownGuard { thread_id, worker: Some(worker) }),
        Ok(Err(error)) => {
            let _ = worker.join();
            Err(error)
        }
        Err(error) => {
            let _ = worker.join();
            Err(Error::Platform(format!("keyboard hook worker exited during startup: {error}")))
        }
    }
}

unsafe extern "system" fn callback(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        // SAFETY: Windows guarantees lparam points to a KBDLLHOOKSTRUCT for
        // HC_ACTION callbacks on a WH_KEYBOARD_LL hook.
        let raw = unsafe { &*(lparam as *const KBDLLHOOKSTRUCT) };
        let extended = raw.flags & LLKHF_EXTENDED != 0;
        let native_code = RawCode::Windows {
            scan_code: raw.scanCode,
            virtual_key: raw.vkCode,
            extended,
        };
        // Use the scan code for Key because it describes the physical key.
        // Keep the virtual key in raw_code for callers which need it.
        let key = keycodes::from_scan_code(raw.scanCode, raw.vkCode, extended);
        let message = wparam as u32;

        CONTEXT.with(|context| {
            let mut context = context.borrow_mut();
            let Some(context) = context.as_mut() else {
                return;
            };

            let kind = if message == WM_KEYDOWN || message == WM_SYSKEYDOWN {
                Some(EventKind::KeyDown(key))
            } else if message == WM_KEYUP || message == WM_SYSKEYUP {
                Some(EventKind::KeyUp(key))
            } else {
                None
            };

            if let Some(kind) = kind {
                let _ = context.sender.send(Event {
                    time: Instant::now(),
                    kind,
                    raw_code: Some(native_code),
                });
            }
        });
    }

    // SAFETY: every observe-only hook must forward the event. A null hook is
    // accepted for low-level hooks and asks Windows to continue the chain.
    unsafe { CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam) }
}
