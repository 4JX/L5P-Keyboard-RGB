mod cli;
mod custom_effect;
mod effects;
mod enums;
mod error;
mod keyboard_utils;
mod profile;
mod storage_trait;

use color_eyre::{Report, Result};

fn main() -> Result<(), Report> {
	color_eyre::install()?;

	// Clear/Hide console if not running via one (Windows specific)
	#[cfg(target_os = "windows")]
	{
		#[link(name = "Kernel32")]
		extern "system" {
			fn GetConsoleProcessList(processList: *mut u32, count: u32) -> u32;
			fn FreeConsole() -> i32;
		}

		fn free_console() -> bool {
			unsafe { FreeConsole() == 0 }
		}

		fn is_console() -> bool {
			unsafe {
				let mut buffer = [0_u32; 1];
				let count = GetConsoleProcessList(buffer.as_mut_ptr(), 1);
				count != 1
			}
		}

		if !is_console() {
			free_console();
		}
	}

	cli::try_cli()?;

	Ok(())
}
