use serde::Deserialize;
use serde_yaml::Value;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;

#[ derive (Deserialize) ]
#[ serde (deny_unknown_fields) ]
pub struct Config {
	pub core: CoreConfig,
	pub programmes: HashMap <String, ProgrammeConfig>,
}

#[ derive (Deserialize) ]
#[ serde (deny_unknown_fields) ]
pub struct CoreConfig {
	pub sleep_millis: u16,
}

#[ derive (Deserialize) ]
#[ serde (deny_unknown_fields) ]
pub struct ProgrammeConfig {
	pub r#type: String,
	pub config: Value,
}

impl Config {

	pub fn load () -> Result <Config, Box <dyn Error>> {
		let mut file = File::open ("config.yaml") ?;
		let config = serde_yaml::from_reader (& mut file) ?;
		Ok (config)
	}

}

