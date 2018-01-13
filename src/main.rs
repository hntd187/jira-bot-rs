extern crate slack;
extern crate config;
extern crate botlib;

use slack::RtmClient;
use config::{Config, File};
use std::process::exit;

use botlib::bot;

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

  let mut handler = bot::SlackHandler::new(&token);
  match RtmClient::login_and_run(&token, &mut handler) {
    Ok(_) => (),
    Err(err) => panic!("Error: {}", err)
  }
}
