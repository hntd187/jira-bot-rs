extern crate json;
extern crate reqwest;

use std::result::Result;
use std::io::Read;

use reqwest::Client;
use reqwest::header;
use reqwest::{Response, Url};

#[allow(unused_must_use)]
fn read_body(response: &mut Response) -> String {
  let mut txt = String::new();
  response.read_to_string(&mut txt);
  return txt;
}

fn jira_request(client: Client, base: &str, rapid_id: &str, sprint_id: &str) -> json::Result<json::JsonValue> {
  let params = [("rapidId", rapid_id), ("sprintId", sprint_id)];
  let heads = header::Headers::new();
  let base_url = match Url::parse_with_params(base, &params) {
    Ok(u) => u,
    Err(e) => panic!("Error parsing url: {:?} with {:?}, Reason: {:?}", base, params.to_vec(), e),
  };
  let base = base_url.join("charts/sprintreport").unwrap();

  let body = client.get(base_url).send().ok()
    .map(|mut r: Response| read_body(&mut r)).unwrap();

  return json::parse(&body);
}

pub fn sprint_report(req: Client, base: &str, rapid_id: &str, sprint_id: &str) -> Result<String, String> {
  return Ok(String::new());
}