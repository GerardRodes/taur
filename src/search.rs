use chrono::{DateTime, Utc, serde::{ts_seconds}};

use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use serde::Deserialize;
pub enum By {
	Name,
	NameDesc,
	Maintainer,
	Depends,
	MakeDepends,
	OptDepends,
	CheckDepends
}

impl By {
	#[must_use] pub fn as_str(&self) -> &'static str {
		match self {
			By::Name=> "name",
			By::NameDesc=> "name-desc",
			By::Maintainer=> "maintainer",
			By::Depends=> "depends",
			By::MakeDepends=> "makedepends",
			By::OptDepends=> "optdepends",
			By::CheckDepends=> "checkdepends",
		}
	}
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Package {
	pub description: Option<String>,
	#[serde(with = "ts_seconds")]
	pub first_submitted: DateTime<Utc>,
	#[serde(with = "ts_seconds")]
	pub last_modified: DateTime<Utc>,
	#[serde(rename = "ID")]
	pub id: u32,
	pub maintainer: Option<String>,
	pub name: String,
	pub num_votes: u32,
	// pub out_of_date: null,
	pub package_base: Option<String>,
	#[serde(rename = "PackageBaseID")]
	pub package_base_id: u32,
	pub popularity: f32,
	#[serde(rename = "URL")]
	pub url: Option<String>,
	#[serde(rename = "URLPath")]
	pub url_path: Option<String>,
	pub version: Option<String>,
}

#[derive(Deserialize, Debug)]
struct SearchResponse {
	results: Vec<Package>,
}

const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b'"').add(b'<').add(b'>').add(b'`');
const BASE_URL: &str = "https://aur.archlinux.org/rpc?v=5&type=search&arg=";

// https://aur.archlinux.org/rpc.php
/// # Errors
/// # Panics
pub async fn search(by: By, search: &str) -> Vec<Package> {
	// "https://aur.archlinux.org/rpc?v=5&type=search"
	let arg = utf8_percent_encode(search, FRAGMENT).to_string();
	let search_url = BASE_URL.to_string() + &arg + "&by=" + by.as_str();

	let reqr = reqwest::get(search_url).await;
	if reqr.is_err() {
		return Vec::new();
	}

	let mut results = reqr.unwrap()
		.json::<SearchResponse>()
		.await
		.unwrap_or(SearchResponse{results: Vec::new()})
		.results;

	results.sort_by(|a, b| b.popularity.total_cmp(&a.popularity));
	results
}