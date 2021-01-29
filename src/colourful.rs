use async_trait::async_trait;

use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;

use rand::Rng;
use rand::prelude::SliceRandom;

use serde::Deserialize;
use serde_yaml::Value;

use std::collections::HashMap;
use std::error::Error;

use crate::HueAll;
use crate::HueClient;
use crate::HueLightState;
use crate::Programme;

#[ derive (Deserialize) ]
#[ serde (deny_unknown_fields, rename_all = "kebab-case") ]
pub struct ColourfulConfig {
	pub interval_time: u16,
	pub transition_time: u16,
	pub lights: Vec <String>,
}

pub struct ColourfulProgramme {
	light_ids: Vec <String>,
	interval_time: Duration,
	transition_time: u16,
	next_run: DateTime <Utc>,
	last_light_id: String,
}

impl ColourfulProgramme {

	pub fn build (
		light_ids_by_name: & HashMap <String, String>,
		config: & Value,
	) -> Result <Box <dyn Programme>, Box <dyn Error>> {

		let config: ColourfulConfig = serde_yaml::from_value (config.clone ()) ?;

		Ok (Box::new (ColourfulProgramme {
			light_ids: config.lights.iter ().map (
				|light_name| light_ids_by_name [light_name].to_string (),
			).collect (),
			interval_time: Duration::milliseconds (config.interval_time as i64 * 100),
			transition_time: config.transition_time,
			next_run: Utc::now (),
			last_light_id: String::new (),
		}))

	}

}

#[ async_trait ]
impl Programme for ColourfulProgramme {

	async fn tick (
		& mut self,
		client: & HueClient,
		all_data: & HueAll,
	) {

		if Utc::now () < self.next_run {
			return;
		}

		self.next_run = self.next_run + self.interval_time;

		let light_id: & str;
		let hue: u16;
		let sat: u8;

		{

			let mut rng = rand::thread_rng ();

			light_id = loop {
				let light_id = self.light_ids.choose (& mut rng).unwrap ();
				if light_id != & self.last_light_id { break light_id }
			};

			hue = rng.gen ();
			sat = rng.gen::<u8> () | 0xc0;

		}

		let light_data = & all_data.lights [light_id];

		println! ("Colourful set {} ({}): hue={} sat={}", light_data.name, light_id, hue, sat);

		if let Err (error) =
			client.set_light_state (& light_id, & HueLightState {
				hue: Some (hue),
				sat: Some (sat),
				transitiontime: Some (self.transition_time),
				.. Default::default ()
			}).await {

			eprintln! ("Error setting light state: {}", error);

		}

		self.last_light_id = light_id.to_string ();

	}

}

