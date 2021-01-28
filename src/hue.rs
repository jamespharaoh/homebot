use reqwest::Client;

use serde::Deserialize;
use serde::Serialize;

use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

pub struct HueClient {
	client: Client,
	hostname: String,
	username: String,
}

impl HueClient {

	pub fn new (
		client: Client,
		hostname: String,
		username: String,
	) -> HueClient {

		HueClient {
			client,
			hostname,
			username,
		}

	}

	pub async fn get_all (
		& self,
	) -> Result <HueAll, Box <dyn Error>> {

		Ok (HueAll {
			lights: self.get_lights ().await ?,
			groups: self.get_groups ().await ?,
		})

	}

	pub async fn get_lights (
		& self,
	) -> Result <HashMap <String, Arc <HueLight>>, Box <dyn Error>> {

		let url = format! (
			"http://{}/api/{}/lights",
			self.hostname,
			self.username,
		);

		let response = self.client.get (& url).send ().await ?;
		let response_body = response.json ().await ?;

		Ok (response_body)

	}

	pub async fn get_light (
		& self,
		light_id: & str,
	) -> Result <HueLight, Box <dyn Error>> {

		let url = format! (
			"http://{}/api/{}/lights/{}",
			self.hostname,
			self.username,
			light_id,
		);

		let response = self.client.get (& url).send ().await ?;
		let response_body = response.json ().await ?;

		Ok (response_body)

	}

	pub async fn get_groups (
		& self,
	) -> Result <HashMap <String, Arc <HueGroup>>, Box <dyn Error>> {

		let url = format! (
			"http://{}/api/{}/groups",
			self.hostname,
			self.username,
		);

		let response = self.client.get (& url).send ().await ?;
		let response_body = response.json ().await ?;

		Ok (response_body)

	}

	pub async fn set_light_state (
		& self,
		light_id: & str,
		light_state: & HueLightState,
	) -> Result <(), Box <dyn Error>> {

		let url = format! (
			"http://{}/api/{}/lights/{}/state",
			self.hostname,
			self.username,
			light_id,
		);

		let response = self.client.put (& url).json (light_state).send ().await ?;

		// TODO check response

		Ok (())

	}

}

#[ derive (Debug) ]
pub struct HueAll {
	pub lights: HashMap <String, Arc <HueLight>>,
	pub groups: HashMap <String, Arc <HueGroup>>,
}

#[ derive (Serialize, Debug, Deserialize) ]
pub struct HueGroup {
	pub action: HueLightState,
	pub lights: Vec <String>,
	pub name: String,
	pub r#type: String,
	pub modelid: Option <String>,
	pub uniqueid: Option <String>,
	pub class: Option <String>,
}

#[ derive (Serialize, Debug, Deserialize) ]
pub struct HueLight {
	pub state: HueLightState,
	pub swupdate: HueLightSwUpdate,
	pub r#type: String,
	pub name: String,
	pub modelid: String,
	pub manufacturername: String,
	pub productname: String,
	pub capabilities: HueLightCapabilities,
	pub config: HueLightConfig,
	pub uniqueid: String,
	pub swversion: String,
}

#[ derive (Serialize, Debug, Default, Deserialize) ]
pub struct HueLightState {
	#[serde (skip_serializing_if = "Option::is_none")] pub on: Option <bool>,
	#[serde (skip_serializing_if = "Option::is_none")] pub bri: Option <u8>,
	#[serde (skip_serializing_if = "Option::is_none")] pub hue: Option <u16>,
	#[serde (skip_serializing_if = "Option::is_none")] pub sat: Option <u8>,
	#[serde (skip_serializing_if = "Option::is_none")] pub xy: Option <(f32, f32)>,
	#[serde (skip_serializing_if = "Option::is_none")] pub ct: Option <u16>,
	#[serde (skip_serializing_if = "Option::is_none")] pub alert: Option <String>,
	#[serde (skip_serializing_if = "Option::is_none")] pub effect: Option <String>,
	#[serde (skip_serializing_if = "Option::is_none")] pub colormode: Option <String>,
	#[serde (skip_serializing_if = "Option::is_none")] pub reachable: Option <bool>,
	#[serde (skip_serializing_if = "Option::is_none")] pub transitiontime: Option <u16>,
	#[serde (skip_serializing_if = "Option::is_none")] pub bri_inc: Option <i16>,
	#[serde (skip_serializing_if = "Option::is_none")] pub sat_inc: Option <i16>,
	#[serde (skip_serializing_if = "Option::is_none")] pub hue_inc: Option <i32>,
	#[serde (skip_serializing_if = "Option::is_none")] pub ct_inc: Option <i32>,
	#[serde (skip_serializing_if = "Option::is_none")] pub xy_inc: Option <i32>,
}

#[ derive (Serialize, Debug, Deserialize) ]
pub struct HueLightSwUpdate {
}

#[ derive (Serialize, Debug, Deserialize) ]
pub struct HueLightCapabilities {
}

#[ derive (Serialize, Debug, Deserialize) ]
pub struct HueLightConfig {
}

