use chrono::Duration;

use futures::try_join;

use std::error::Error;
use std::process;

use tokio::time as tokio_time;

use crate::colourful::*;
use crate::config::*;
use crate::dailyroutine::*;
use crate::hue::*;
use crate::programme::*;

mod colourful;
mod config;
mod dailyroutine;
mod hue;
mod programme;

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

	let programme_manager = ProgrammeManager::new (
		& config,
		& all_data,
	) ?;

	try_join! (
		main_loop (
			& config,
			& client,
			programme_manager.clone (),
		),
	) ?;

	Ok (())

}

async fn main_loop (
	config: & Config,
	client: & HueClient,
	programme_manager: ProgrammeManager,
) -> Result <(), Box <dyn Error>> {

	loop {

		let all_data = client.get_all ().await ?;

		programme_manager.tick (
			& client,
			& all_data,
		).await;

		tokio_time::sleep (
			Duration::milliseconds (
				config.core.sleep_millis as i64,
			).to_std ().unwrap (),
		).await;

	}

}

