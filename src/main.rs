#![feature(box_patterns, try_trait)]
extern crate slack;
extern crate config;
extern crate reqwest;

mod jira_bot;

use slack::RtmClient;

use config::{Config, File};

fn main() {

  let mut cfg = Config::default();
  cfg.merge(File::with_name("conf/config.yml"));

  let token = cfg.get_str("slack_key").unwrap();
  let base_url = cfg.get_str("jira_base").unwrap();

  let mut handler = jira_bot::SlackHandler::new(&token, &base_url);
  let r = RtmClient::login_and_run(&token, &mut handler);
  match r {
    Ok(_) => {}
    Err(err) => panic!("Error: {}", err),
  }
}
