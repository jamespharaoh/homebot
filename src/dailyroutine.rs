use async_trait::async_trait;

use chrono::Local;
use chrono::NaiveTime;

use serde::Deserialize;
use serde_yaml::Value;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use crate::HueAll;
use crate::HueClient;
use crate::HueLightState;
use crate::Programme;

#[ derive (Deserialize) ]
#[ serde (deny_unknown_fields, rename_all = "kebab-case") ]
pub struct DailyRoutineConfig {
	pub transition_times: DailyRoutineTransitionTimesConfig,
	pub brightness: HashMap <String, (u8, u8, u8, u8)>,
	pub colour_temperature: HashMap <String, (u16, u16, u16, u16)>,
	pub light_groups: HashMap <String, Vec <String>>,
}

#[ derive (Deserialize) ]
#[ serde (deny_unknown_fields, rename_all = "kebab-case") ]
pub struct DailyRoutineTransitionTimesConfig {
	pub morning: (NaiveTime, NaiveTime),
	pub evening: (NaiveTime, NaiveTime),
	pub bedtime_1: (NaiveTime, NaiveTime),
	pub bedtime_2: (NaiveTime, NaiveTime),
}

#[ derive (Clone) ]
pub struct DailyRoutineProgramme {
	inner: Arc <DailyRoutineInner>,
}

pub struct DailyRoutineInner {
	name: String,
	transition_times: Vec <NaiveTime>,
	light_groups: Vec <DailyRoutineLightGroup>,
}

pub struct DailyRoutineLightGroup {
	name: String,
	light_ids: Vec <String>,
	brightnesses: Vec <u8>,
	colour_temperatures: Vec <u16>,
}

impl DailyRoutineProgramme {

	fn clone (& self) -> Box <dyn Programme> {
		Box::new (DailyRoutineProgramme {
			inner: self.inner.clone (),
		})
	}

	pub fn build (
		light_ids_by_name: & HashMap <String, String>,
		name: String,
		config: & Value,
	) -> Result <Box <dyn Programme>, Box <dyn Error>> {

		let config: DailyRoutineConfig =
			serde_yaml::from_value (config.clone ()) ?;

		let transition_times = vec! [
			NaiveTime::from_hms (0, 0, 0),
			config.transition_times.morning.0,
			config.transition_times.morning.1,
			config.transition_times.evening.0,
			config.transition_times.evening.1,
			config.transition_times.bedtime_1.0,
			config.transition_times.bedtime_1.1,
			config.transition_times.bedtime_2.0,
			config.transition_times.bedtime_2.1,
			NaiveTime::from_hms_milli (0, 0, 0, 1000),
		];

		let mut light_groups = Vec::new ();

		for (group_name, group_lights) in config.light_groups.iter () {

			let light_ids = group_lights.iter ().map (
				|light_name| light_ids_by_name [light_name].clone (),
			).collect ();

			let group_brightnesses = config.brightness [group_name];
			let group_colour_temperatures = config.colour_temperature [group_name];

			let brightnesses = vec! [
				group_brightnesses.3,
				group_brightnesses.3,
				group_brightnesses.0,
				group_brightnesses.0,
				group_brightnesses.1,
				group_brightnesses.1,
				group_brightnesses.2,
				group_brightnesses.2,
				group_brightnesses.3,
				group_brightnesses.3,
			];

			let colour_temperatures = vec! [
				group_colour_temperatures.3,
				group_colour_temperatures.3,
				group_colour_temperatures.0,
				group_colour_temperatures.0,
				group_colour_temperatures.1,
				group_colour_temperatures.1,
				group_colour_temperatures.2,
				group_colour_temperatures.2,
				group_colour_temperatures.3,
				group_colour_temperatures.3,
			];

			light_groups.push (DailyRoutineLightGroup {
				name: group_name.clone (),
				light_ids,
				brightnesses,
				colour_temperatures,
			});

		}

		Ok (Box::new (DailyRoutineProgramme {
			inner: Arc::new (DailyRoutineInner {
				name,
				transition_times,
				light_groups,
			}),
		}))

	}

}

#[ async_trait ]
impl Programme for DailyRoutineProgramme {

	fn clone (& self) -> Box <dyn Programme> {
		Box::new (DailyRoutineProgramme {
			inner: self.inner.clone (),
		})
	}

	async fn activate (
		& self,
		_client: & HueClient,
		_all_data: & HueAll,
	) {
	}

	async fn deactivate (
		& self,
		_client: & HueClient,
		_all_data: & HueAll,
	) {
	}

	async fn tick (
		& self,
		client: & HueClient,
		all_data: & HueAll,
	) {

		let inner = self.inner.as_ref ();

		let now = Local::now ();
		let date = now.date ();
		let time = now.time ();

		let index = inner.transition_times.iter ().skip (1).take_while (
			|transition_time| ** transition_time <= time
		).count ();

		let start_time = inner.transition_times [index];
		let end_time = inner.transition_times [index + 1];

		let start_instant = date.and_time (start_time).unwrap ();
		let end_instant = date.and_time (end_time).unwrap ();

		let total_seconds = end_instant.timestamp () - start_instant.timestamp ();
		let elapsed_seconds = now.timestamp () - start_instant.timestamp ();
		let progress = elapsed_seconds * 0x10000 / total_seconds;

		for light_group in inner.light_groups.iter () {

			fn interpolate (range: (i64, i64), progress: i64) -> i64 {
				range.0 + (range.1 - range.0) * progress / 0x10000
			}

			let new_brightness = interpolate (
				(
					light_group.brightnesses [index] as i64,
					light_group.brightnesses [index + 1] as i64,
				),
				progress,
			) as u8;

			let new_colour_temperature = interpolate (
				(
					light_group.colour_temperatures [index] as i64,
					light_group.colour_temperatures [index + 1] as i64,
				),
				progress,
			) as u16;

			for light_id in light_group.light_ids.iter () {

				let light_data = & all_data.lights [light_id];
				let mut new_state: HueLightState = Default::default ();

				if ! light_data.state.on.unwrap () {
					if index == 1 {
						new_state.on = Some (true);
					} else {
						continue;
					}
				}

				if ! light_data.state.on.unwrap ()
					|| (light_data.state.ct.is_some ()
						&& new_brightness != light_data.state.bri.unwrap ()) {

					println! (
						"[{}] {} ({}) brightness from {} to {}",
						inner.name,
						light_data.name,
						light_id,
						light_data.state.bri.unwrap_or (0),
						new_brightness,
					);

					new_state.bri = Some (new_brightness);

				}

				if ! light_data.state.on.unwrap ()
					|| (light_data.state.colormode.as_ref ().map (String::as_str) == Some ("ct")
						&& light_data.state.ct.unwrap () != new_colour_temperature) {

					println! (
						"[{}] {} ({}) colour temperature from {} to {}",
						inner.name,
						light_data.name,
						light_id,
						light_data.state.ct.unwrap_or (0),
						new_colour_temperature,
					);

					new_state.ct = Some (new_colour_temperature);

				}

				if new_state.on.is_some () || new_state.bri.is_some ()
					|| new_state.ct.is_some () {

					if let Err (error) = client.set_light_state (
						& light_id,
						& new_state,
					).await {
						println! ("Error setting light state: {}", error);
						continue;
					}

				}

			}

		}

	}

}

