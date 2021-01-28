use async_trait::async_trait;

use chrono::DateTime;
use chrono::Duration;
use chrono::Local;
use chrono::NaiveTime;
use chrono::Utc;

use futures;
use futures::future;

use hue::*;

use rand;
use rand::prelude::SliceRandom as _;
use rand::Rng;

use std::collections::HashMap;
use std::error::Error;
use std::process;

use tokio::time as tokio_time;

mod hue;

const HOSTNAME: & str = "10.109.131.103";
const USERNAME: & str = "XN8wjhtZLtQoy4Aae2fA95A38s59Cqht8kd5yqxN";
const SLEEP_TIME: u32 = 1;

#[tokio::main]
async fn main () {
	if let Err (error) = main_inner ().await {
		eprintln! ("Error: {}", error);
		process::exit (1);
	}
}

async fn main_inner () -> Result <(), Box <dyn Error>> {

	let client = HueClient::new (
		reqwest::Client::new (),
		HOSTNAME.to_string (),
		USERNAME.to_string (),
	);

	let all_data = client.get_all ().await ?;

	let light_ids_by_name: HashMap <String, String> = all_data.lights.iter ().map (
		|(light_id, light_data)| (
			light_data.name.to_string (),
			light_id.to_string (),
		),
	).collect ();

	let programme_specs: Vec <Box <dyn ProgrammeSpec>> = vec! [

		// colourful effect

		Box::new (ColourfulProgrammeSpec {
			light_names: vec! [
				"Lantern 1",
				"Lantern 2",
				"Kitchen 1",
				"Kitchen 2",
				"Kitchen 3",
				"Kitchen 4",
			].iter ().map (|s| s.to_string ()).collect (),
			interval_time: 40,
			transition_time: 20,
		}),

		Box::new (ColourfulProgrammeSpec {
			light_names: vec! [
				"Bedroom uplighter 1",
				"Bedroom uplighter 2",
			].iter ().map (|s| s.to_string ()).collect (),
			interval_time: 80,
			transition_time: 20,
		}),

		// sunset

		Box::new (FadeAtNightProgrammeSpec {
			light_names: vec! [
				"Bedroom ceiling",
				"Living room ceiling",
			].iter ().map (|s| s.to_string ()).collect (),
			start_time: NaiveTime::from_hms (17, 0, 0),
			end_time: NaiveTime::from_hms (18, 0, 0),
			bri_range: (255, 96),
			ct_range: (153, 300),
		}),

		Box::new (FadeAtNightProgrammeSpec {
			light_names: vec! [
				"Bathroom 1",
				"Bathroom 2",
				"Bedroom uplighter 1",
				"Bedroom uplighter 2",
				"Hall 1",
				"Hall 2",
				"Kitchen 1",
				"Kitchen 2",
				"Kitchen 3",
				"Kitchen 4",
				"Lantern 1",
				"Lantern 2",
			].iter ().map (|s| s.to_string ()).collect (),
			start_time: NaiveTime::from_hms (17, 0, 0),
			end_time: NaiveTime::from_hms (18, 0, 0),
			bri_range: (255, 128),
			ct_range: (153, 300),
		}),

		// bedtime

		Box::new (FadeAtNightProgrammeSpec {
			light_names: vec! [
				"Bedroom ceiling",
				"Living room ceiling",
			].iter ().map (|s| s.to_string ()).collect (),
			start_time: NaiveTime::from_hms (21, 0, 0),
			end_time: NaiveTime::from_hms (22, 0, 0),
			bri_range: (96, 24),
			ct_range: (300, 400),
		}),

		Box::new (FadeAtNightProgrammeSpec {
			light_names: vec! [
				"Bathroom 1",
				"Bathroom 2",
				"Bedroom uplighter 1",
				"Bedroom uplighter 2",
				"Hall 1",
				"Hall 2",
				"Kitchen 1",
				"Kitchen 2",
				"Kitchen 3",
				"Kitchen 4",
			].iter ().map (|s| s.to_string ()).collect (),
			start_time: NaiveTime::from_hms (21, 0, 0),
			end_time: NaiveTime::from_hms (22, 0, 0),
			bri_range: (128, 32),
			ct_range: (300, 400),
		}),

		Box::new (FadeAtNightProgrammeSpec {
			light_names: vec! [
				"Lantern 1",
				"Lantern 2",
			].iter ().map (|s| s.to_string ()).collect (),
			start_time: NaiveTime::from_hms (21, 0, 0),
			end_time: NaiveTime::from_hms (22, 0, 0),
			bri_range: (128, 64),
			ct_range: (153, 153),
		}),

	];

	let mut programmes: Vec <Box <dyn Programme>> = programme_specs.iter ().map (
		|programme_spec| programme_spec.build (& light_ids_by_name),
	).collect ();

	loop {

		let all_data = client.get_all ().await ?;

		let mut programme_futs = Vec::new ();

		for programme in programmes.iter_mut () {

			programme_futs.push (
				programme.tick (
					& client,
					& all_data,
				),
			);

		}

		future::join_all (programme_futs).await;

		tokio_time::sleep (
			Duration::milliseconds (SLEEP_TIME as i64 * 100).to_std ().unwrap (),
		).await;

	}

}

trait ProgrammeSpec {

	fn build (
		& self,
		light_ids_by_name: & HashMap <String, String>,
	) -> Box <dyn Programme>;

}

#[ async_trait ]
trait Programme {

	async fn tick (
		& mut self,
		client: & HueClient,
		all_data: & HueAll,
	);

}

struct FadeAtNightProgrammeSpec {
	light_names: Vec <String>,
	start_time: NaiveTime,
	end_time: NaiveTime,
	bri_range: (u8, u8),
	ct_range: (u16, u16),
}

impl ProgrammeSpec for FadeAtNightProgrammeSpec {

	fn build (
		& self,
		light_ids_by_name: & HashMap <String, String>,
	) -> Box <dyn Programme> {

		Box::new (FadeAtNightProgramme {
			lights: self.light_names.iter ().map (
				|light_name| FadeAtNightLight {
					id: light_ids_by_name [light_name].to_owned (),
				},
			).collect (),
			start_time: self.start_time,
			end_time: self.end_time,
			bri_range: self.bri_range,
			ct_range: self.ct_range,
		})

	}

}

struct FadeAtNightProgramme {
	lights: Vec <FadeAtNightLight>,
	start_time: NaiveTime,
	end_time: NaiveTime,
	bri_range: (u8, u8),
	ct_range: (u16, u16),
}

struct FadeAtNightLight {
	id: String,
}

#[ async_trait ]
impl Programme for FadeAtNightProgramme {

	async fn tick (
		& mut self,
		client: & HueClient,
		all_data: & HueAll,
	) {

		let now = Local::now ();
		let date = now.date ();
		let time = now.time ();

		if time < self.start_time || self.end_time < time {
			return;
		}

		for light in self.lights.iter_mut () {

			let light_data = match all_data.lights.get (& light.id) {
				Some (value) => value,
				None => continue,
			};

			if ! light_data.state.on.unwrap () {
				continue;
			}

			let start_instant = date.and_time (self.start_time).unwrap ();
			let end_instant = date.and_time (self.end_time).unwrap ();

			let total_seconds = end_instant.timestamp () - start_instant.timestamp ();
			let elapsed_seconds = now.timestamp () - start_instant.timestamp ();
			let remaining_seconds = end_instant.timestamp () - now.timestamp ();
			let progress = elapsed_seconds * 0x10000 / total_seconds;

			let mut new_state: HueLightState = Default::default ();

			// reduce brightness

			let bri_diff = self.bri_range.1 as i64 - self.bri_range.0 as i64;

			let new_bri = (
				(self.bri_range.0 as i64)
				+ bri_diff * progress as i64 / 0x10000
			) as u8;

			if new_bri < light_data.state.bri.unwrap () {

				println! (
					"Reduce {} ({}) bri from {} to {}",
					light_data.name,
					light.id,
					light_data.state.bri.unwrap (),
					new_bri,
				);

				new_state.bri = Some (new_bri);

			}

			// increase colour temperature

			let ct_diff = self.ct_range.1 as i64 - self.ct_range.0 as i64;

			let new_ct = (
				(self.ct_range.0 as i64)
				+ ct_diff * progress as i64 / 0x10000
			) as u16;

			if light_data.state.colormode.as_ref ().unwrap () == "ct"
				&& light_data.state.ct.unwrap () < new_ct {

				println! (
					"Increase {} ({}) ct from {} to {}",
					light_data.name,
					light.id,
					light_data.state.ct.unwrap (),
					new_ct,
				);

				new_state.ct = Some (new_ct);

			}

			if new_state.bri.is_some () || new_state.ct.is_some () {

				if let Err (error) = client.set_light_state (
					& light.id,
					& new_state,
				).await {
					println! ("Error setting light state: {}", error);
					continue;
				}

			}

		}

	}

}

struct ColourfulProgrammeSpec {
	light_names: Vec <String>,
	interval_time: u16,
	transition_time: u16,
}

impl ProgrammeSpec for ColourfulProgrammeSpec {

	fn build (
		& self,
		light_ids_by_name: & HashMap <String, String>,
	) -> Box <dyn Programme> {

		Box::new (ColourfulProgramme {
			light_ids: self.light_names.iter ().map (
				|light_name| light_ids_by_name [light_name].to_string (),
			).collect (),
			interval_time: Duration::milliseconds (self.interval_time as i64 * 100),
			transition_time: self.transition_time,
			next_run: Utc::now (),
			last_light_id: String::new (),
		})

	}

}

struct ColourfulProgramme {
	light_ids: Vec <String>,
	interval_time: Duration,
	transition_time: u16,
	next_run: DateTime <Utc>,
	last_light_id: String,
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

