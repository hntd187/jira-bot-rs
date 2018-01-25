#[macro_use]
extern crate botlib;
extern crate chrono;
#[macro_use]
extern crate clap;
extern crate config;
extern crate fern;
extern crate log;
extern crate slack;
extern crate yaml_rust;

use std::fmt::Arguments;
use std::io::stdout;

use chrono::Local;
use clap::{App, Arg};
use config::{Config, File};
use log::Record;
use slack::RtmClient;

use botlib::bot::SlackHandler;
use botlib::jira::Jira;

const DATE_FMT: &str = "[%Y-%m-%d] [%H:%M:%S]";

fn format_record(o: fern::FormatCallback, m: &Arguments, r: &Record) {
  o.finish(format_args!("{} [{}] [{}]: {}", Local::now().format(DATE_FMT), r.target(), r.level(), m))
}

fn setup_logging() -> Result<(), fern::InitError> {
  fern::Dispatch::new().format(format_record).level(log::LevelFilter::Info).chain(stdout()).apply()?;
  Ok(())
}

fn main() {
  let args = App::new("JIRA Report CLI")
    .version(crate_version!())
    .author(crate_authors!())
    .arg(Arg::with_name("config").long("cfg").short("c").default_value("conf/config.yml"))
    .arg(Arg::with_name("users").long("users").short("u").default_value("conf/users.yml"));

  setup_logging().expect("Could not setup logging.");

  let matches = args.get_matches();
  let mut cfg = Config::default();

  let cfg_file = matches.value_of("config").unwrap();
  let users_file = matches.value_of("users").unwrap();
  cfg.merge(File::with_name(cfg_file));

  let token = cfg.get_str("slack_key").expect("No Slack Key in config.yml");
  let jira = Jira::new(&cfg, users_file);

  let mut handler = SlackHandler::new(jira);
  check!(RtmClient::login_and_run(&token, &mut handler));
}
