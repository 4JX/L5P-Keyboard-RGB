mod cli;
mod effects;
mod enums;
mod error;
mod gui;
mod profile;
mod util;

use color_eyre::{eyre::eyre, Result};
use effects::EffectManager;
use eframe::{epaint::Vec2, IconData};
use gui::App;
use util::is_unique_instance;

const WINDOW_SIZE: Vec2 = Vec2::new(500., 400.);

fn main() -> Result<()> {
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

	let is_unique = is_unique_instance();

	let cli_output = cli::try_cli(is_unique).map_err(|err| eyre!("{:?}", err))?;

	match cli_output.start_gui_maybe_hidden {
		Some(hide_window) => {
			let app_icon = load_icon_data(include_bytes!("../res/trayIcon.ico"));
			let native_options = eframe::NativeOptions {
				initial_window_size: Some(WINDOW_SIZE),
				min_window_size: Some(WINDOW_SIZE),
				max_window_size: Some(WINDOW_SIZE),
				icon_data: Some(app_icon),
				..eframe::NativeOptions::default()
			};

			let app = App::new(cli_output.output, hide_window, is_unique);

			eframe::run_native("Legion RGB", native_options, Box::new(|cc| Box::new(app.init(cc))));

			Ok(())
		}
		None => {
			let mut effect_manager = EffectManager::new(effects::OperationMode::Cli).unwrap();

			match cli_output.output {
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
}

#[must_use]
fn load_icon_data(image_data: &[u8]) -> IconData {
	let image = image::load_from_memory(image_data).unwrap();
	let image_buffer = image.to_rgba8();
	let pixels = image_buffer.into_flat_samples().samples;

	IconData {
		rgba: pixels,
		width: image.width(),
		height: image.height(),
	}
}
