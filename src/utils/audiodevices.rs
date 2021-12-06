//This file is adapted from https://github.com/dheijl/swyh-rs/blob/44676eb543d4238311c87b68858589e60d0debe7/src/utils/audiodevices.rs

use cpal::traits::{DeviceTrait, HostTrait};
use flume::Sender;
use parking_lot::Once;

pub fn get_output_audio_devices() -> Vec<cpal::Device> {
	let mut result: Vec<cpal::Device> = Vec::new();

	let available_hosts = cpal::available_hosts();

	for host_id in available_hosts {
		let host = cpal::host_from_id(host_id).unwrap();

		// let default_out = host.default_output_device().map(|e| e.name().unwrap());

		let devices = host.devices().unwrap();

		for (_device_index, device) in devices.enumerate() {
			// Output configs
			let mut output_configs = match device.supported_output_configs() {
				Ok(f) => f.peekable(),
				Err(_e) => {
					continue;
				}
			};
			if output_configs.peek().is_some() {
				// for (config_index, config) in output_configs.enumerate() {}
			}
			// use only device with default config
			if let Ok(_conf) = device.default_output_config() {
				result.push(device);
			}
		}
	}

	result
}

pub fn get_default_audio_output_device() -> Option<cpal::Device> {
	// audio hosts
	let _available_hosts = cpal::available_hosts();
	let default_host = cpal::default_host();
	default_host.default_output_device()
}

pub fn capture_output_audio(device: &cpal::Device, rms_sender: Sender<Vec<f32>>) -> Option<cpal::Stream> {
	println!("capturing output");
	let audio_cfg = device.default_output_config().expect("No default output config found");

	let mut f32_samples: Vec<f32> = Vec::with_capacity(16384);
	match audio_cfg.sample_format() {
		cpal::SampleFormat::F32 => match device.build_input_stream(&audio_cfg.config(), move |data, _: &_| wave_reader::<f32>(data, &mut f32_samples, &rms_sender), capture_err_fn) {
			Ok(stream) => Some(stream),
			Err(_e) => None,
		},
		cpal::SampleFormat::I16 => match device.build_input_stream(&audio_cfg.config(), move |data, _: &_| wave_reader::<i16>(data, &mut f32_samples, &rms_sender), capture_err_fn) {
			Ok(stream) => Some(stream),
			Err(_e) => None,
		},
		cpal::SampleFormat::U16 => match device.build_input_stream(&audio_cfg.config(), move |data, _: &_| wave_reader::<u16>(data, &mut f32_samples, &rms_sender), capture_err_fn) {
			Ok(stream) => Some(stream),
			Err(_e) => None,
		},
	}
}

fn capture_err_fn(err: cpal::StreamError) {
	panic!("{}", err);
}

fn wave_reader<T>(samples: &[T], f32_samples: &mut Vec<f32>, sender: &Sender<Vec<f32>>)
where
	T: cpal::Sample,
{
	static INITIALIZER: Once = Once::new();
	INITIALIZER.call_once(|| {});
	f32_samples.clear();
	f32_samples.extend(samples.iter().map(cpal::Sample::to_f32));

	sender.send(f32_samples.clone()).unwrap();
}
