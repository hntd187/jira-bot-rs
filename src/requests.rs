#[macro_use]
extern crate botlib;
#[macro_use]
extern crate clap;
extern crate config;

use clap::*;
use config::Config;
use config::File;

use botlib::jira::Jira;

fn main() {
  let args = App::new("JIRA Report CLI")
    .version(crate_version!())
    .author(crate_authors!())
    .subcommand(SubCommand::with_name("burndown").about("Generates a burndown chart from the current sprint"))
    .subcommand(
      SubCommand::with_name("report")
        .about("Generates a sprint text report")
        .arg(Arg::with_name("verbose").short("v").long("verbose").help("Include detailed issue breakdown."))
    )
    .arg(Arg::with_name("sprint_id").short("s").long("sprint_id").required(true).value_name("SPRINT ID").takes_value(true))
    .arg(Arg::with_name("rapid_id").short("r").long("rapid_id").required(true).value_name("RAPID ID").takes_value(true))
    .arg(Arg::with_name("config").long("cfg").short("c").default_value("conf/config.yml"))
    .arg(Arg::with_name("users").long("users").short("u").default_value("conf/users.yml"));

  let matches = args.get_matches();

  let mut cfg = Config::default();
  let sprint_id = matches.value_of("sprint_id").unwrap();
  let rapid_id = matches.value_of("rapid_id").unwrap();
  let cfg_file = matches.value_of("config").unwrap();
  let users_file = matches.value_of("users").unwrap();

  cfg.merge(File::with_name(cfg_file));

  let jira = Jira::new(&cfg, users_file);

  match matches.subcommand_name() {
    Some("report") => {
      let report = check!(jira.sprint_report(rapid_id, sprint_id));
      println!("{}", report)
    }
    None => println!("uhh...I haven't implemented this yet :("),
    _ => println!("You must specify a subcommand, either burndown or report")
  }
}
