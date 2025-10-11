//! Windows ANSI / Color capability management
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};

static COLORS_ENABLED: AtomicBool = AtomicBool::new(true);
static INIT: OnceCell<()> = OnceCell::new();

pub fn init(no_color: bool) {
    if INIT.get().is_some() {
        return;
    }
    INIT.set(()).ok();
    if no_color || std::env::var("NO_COLOR").is_ok() {
        COLORS_ENABLED.store(false, Ordering::SeqCst);
        return;
    }
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
            STD_OUTPUT_HANDLE,
        };
        unsafe {
            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle != 0 && handle != (-1isize) as isize {
                let mut mode: u32 = 0;
                if GetConsoleMode(handle, &mut mode) != 0 {
                    let _ = SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
                }
            }
        }
    }
}

pub fn colors_enabled() -> bool {
    COLORS_ENABLED.load(Ordering::SeqCst)
}

pub fn maybe_strip(s: String) -> String {
    if colors_enabled() {
        return s;
    }
    let vec = strip_ansi_escapes::strip(s.as_bytes());
    String::from_utf8_lossy(&vec).to_string()
}

// Lightweight internal ANSI stripper if crate unavailable (fallback)
// We'll provide feature gating: if strip-ansi-escapes not present, simple regex removal.
#[allow(dead_code)]
fn _basic_strip(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut bytes = input.bytes().peekable();
    while let Some(b) = bytes.next() {
        if b == 0x1B {
            // ESC
            // consume until 'm' or end
            while let Some(nb) = bytes.next() {
                if nb == b'm' {
                    break;
                }
            }
        } else {
            out.push(b as char);
        }
    }
    out
}
