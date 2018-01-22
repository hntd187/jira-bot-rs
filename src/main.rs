extern crate slack;
extern crate config;
#[macro_use]
extern crate botlib;

use slack::RtmClient;
use config::{Config, File};

use botlib::bot;

fn main() {
  let mut cfg = Config::default();
  cfg.merge(File::with_name("conf/config.yml"));
  let token = cfg.get_str("slack_key").expect("No Slack Key in config.yml");

  let mut handler = bot::SlackHandler::new(&token);
  check!(RtmClient::login_and_run(&token, &mut handler));
}
