mod cli;
mod effects;
mod enums;
mod error;
mod profile;
mod storage_trait;

use color_eyre::{Report, Result};
use effects::EffectManager;

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

	let effect_manager_result = EffectManager::new();

	let output = cli::try_cli()?;

	if output.start_gui {
		panic!("Unimplemented");
	} else {
		let effect_manager = effect_manager_result.unwrap();

		match output.output {
			cli::CliOutputType::Profile(profile) => {
				effect_manager.set_profile(profile);
				effect_manager.join_and_exit();
				Ok(())
			}
			cli::CliOutputType::Custom(effect) => {
				effect_manager.custom_effect(effect);
				effect_manager.join_and_exit();
				Ok(())
			}
			cli::CliOutputType::Exit => Ok(()),
		}
	}
}
