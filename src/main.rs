use async_trait::async_trait;

use chrono::Duration;

use futures;
use futures::future;

use config::*;
use hue::*;

use std::collections::HashMap;
use std::error::Error;
use std::process;

use tokio::time as tokio_time;

use crate::colourful::ColourfulProgramme;
use crate::dailyroutine::DailyRoutineProgramme;

mod colourful;
mod config;
mod dailyroutine;
mod hue;

const HOSTNAME: & str = "10.109.131.103";
const USERNAME: & str = "XN8wjhtZLtQoy4Aae2fA95A38s59Cqht8kd5yqxN";

#[tokio::main]
async fn main () {
	if let Err (error) = main_inner ().await {
		eprintln! ("Error: {}", error);
		process::exit (1);
	}
}

async fn main_inner () -> Result <(), Box <dyn Error>> {

	let config = Config::load () ?;

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

	let mut programmes: Vec <Box <dyn Programme>> = config.programmes.iter ().map (
		|(_programme_name, programme_config)| Ok (
			match programme_config.r#type.as_str () {
				"colourful" => ColourfulProgramme::build (& light_ids_by_name, & programme_config.config) ?,
				"daily-routine" => DailyRoutineProgramme::build (& light_ids_by_name, & programme_config.config) ?,
				_ => return Err (format! ("Programme type invalid: {}", programme_config.r#type).into ()),
			},
		),
	).collect::<Result <Vec <Box <dyn Programme>>, Box <dyn Error>>> () ?;

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
			Duration::milliseconds (config.core.sleep_millis as i64).to_std ().unwrap (),
		).await;

	}

}

#[ async_trait ]
trait Programme {

	async fn tick (
		& mut self,
		client: & HueClient,
		all_data: & HueAll,
	);

}

