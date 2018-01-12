#![feature(box_patterns, try_trait)]
extern crate slack;
extern crate config;
extern crate reqwest;

mod mods;

use mods::bot;

use slack::RtmClient;

use config::{Config, File};
use std::process::exit;

fn get_key(cfg: &Config, key: &str) -> String {
  match cfg.get_str(key) {
    Ok(v) => v,
    Err(e) => {
      eprintln!("Cannot get \"{}\", Reason: {}", key, e);
      exit(1);
    }
  }
}

fn main() {
  let mut cfg = Config::default();
  cfg.merge(File::with_name("conf/config.yml"));

  let token = get_key(&cfg, "slack_key");
  let base_url = get_key(&cfg, "jira_base");

  let mut handler = bot::SlackHandler::new(&token, &base_url);
  match RtmClient::login_and_run(&token, &mut handler) {
    Ok(_) => (),
    Err(err) => panic!("Error: {}", err)
  }
}
