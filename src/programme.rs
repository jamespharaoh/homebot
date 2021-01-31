use async_trait::async_trait;

use std::collections::HashMap;
use std::error::Error;

use std::sync::Arc;
use std::sync::Mutex;

use crate::*;

#[ derive (Clone) ]
pub struct ProgrammeManager {
	inner: Arc <ProgrammeManagerInner>,
}

struct ProgrammeManagerInner {
	programmes: HashMap <String, Box <dyn Programme>>,
	state: Mutex <ProgrammeManagerState>,
}

struct ProgrammeManagerState {
}

impl ProgrammeManager {

	pub fn new (
		config: & Config,
		all_data: & HueAll,
	) -> Result <ProgrammeManager, Box <dyn Error>> {

		let light_ids_by_name: HashMap <String, String> = all_data.lights.iter ().map (
			|(light_id, light_data)| (
				light_data.name.to_string (),
				light_id.to_string (),
			),
		).collect ();

		let programmes = config.programmes.iter ().map (
			|(programme_name, programme_config)| Ok ((
				programme_name.clone (),
				match programme_config.r#type.as_str () {
					"colourful" => ColourfulProgramme::build (& light_ids_by_name, programme_name.clone (), & programme_config.config) ?,
					"daily-routine" => DailyRoutineProgramme::build (& light_ids_by_name, programme_name.clone (), & programme_config.config) ?,
					_ => return Err (format! ("Programme type invalid: {}", programme_config.r#type).into ()),
				},
			)),
		).collect::<Result <HashMap <String, Box <dyn Programme>>, Box <dyn Error>>> () ?;

		Ok (ProgrammeManager {
			inner: Arc::new (ProgrammeManagerInner {
				programmes,
				state: Mutex::new (ProgrammeManagerState {
				}),
			}),
		})

	}

	pub async fn tick (
		& self,
		client: & HueClient,
		all_data: & HueAll,
	) {

		let inner = self.inner.as_ref ();

		for programme in inner.programmes.values () {
			programme.tick (client, all_data).await
		}

	}

}

#[ async_trait ]
pub trait Programme {

	fn clone (& self) -> Box <dyn Programme>;

	async fn activate (
		& self,
		client: & HueClient,
		all_data: & HueAll,
	);

	async fn deactivate (
		& self,
		client: & HueClient,
		all_data: & HueAll,
	);

	async fn tick (
		& self,
		client: & HueClient,
		all_data: & HueAll,
	);

}

