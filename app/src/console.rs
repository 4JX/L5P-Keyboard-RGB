// Carefully stolen from
// https://www.reddit.com/r/rust/comments/e7slkw/how_to_show_and_hide_a_terminal_window/
// https://github.com/Freaky/Compactor/blob/1b4e7142e3cd732689c012eb796f12ebb5b166ef/src/console.rs

// Helper functions for handling the Windows console from a GUI context.
//
// Windows subsystem applications must explicitly attach to an existing console
// before stdio works, and if not available, create their own if they wish to
// print anything.
//
// These functions enable that, primarily for the purposes of displaying Rust
// panics.

use winapi::um::consoleapi::{AllocConsole, GetConsoleMode, SetConsoleMode};
use winapi::um::processenv::GetStdHandle;
use winapi::um::winbase::STD_OUTPUT_HANDLE;
use winapi::um::wincon::{AttachConsole, FreeConsole, GetConsoleWindow, ATTACH_PARENT_PROCESS};
use winapi::{shared::minwindef::DWORD, um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING};

/// Check if we're attached to an existing Windows console
pub fn is_attached() -> bool {
    unsafe { !GetConsoleWindow().is_null() }
}

/// Try to attach to an existing Windows console, if necessary.
///
/// It's normally a no-brainer to call this - it just makes println! and friends
/// work as expected, without cluttering the screen with a console in the general
/// case.
pub fn attach() -> bool {
    if is_attached() {
        return true;
    }

    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) != 0 }
}

/// Try to attach to a console, and if not, allocate ourselves a new one.
pub fn alloc() -> bool {
    if attach() {
        return true;
    }

    unsafe { AllocConsole() != 0 }
}

/// Free any allocated console, if any.
pub fn free() {
    unsafe { FreeConsole() };
}

/// Enable ANSI escape codes for color output in the console.
pub fn enable_ansi_support() -> bool {
    let handle = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
    let mut mode: DWORD = 0;

    // Get the current console mode
    if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
        return false; // Failed to get console mode
    }

    // Enable virtual terminal processing
    mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;

    // Set the new console mode
    unsafe { SetConsoleMode(handle, mode) != 0 }
}

pub fn alloc_with_color_support() -> bool {
    if alloc() {
        return enable_ansi_support();
    }

    false // Failed to allocate a console
}
