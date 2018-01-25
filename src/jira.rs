use std::error::Error;
use std::fmt;
use std::fs::File;
use std::io::Read;

use chrono::Duration;
use chrono::prelude::*;
use config::Config;
use json::JsonValue;
use reqwest::Url;
use yaml_rust::{Yaml, YamlLoader};

use http;

const DATE_FMT: &str = "%d/%b/%y %I:%M %p";

#[derive(Default)]
pub struct Jira {
  cfg: Config,
  users: Vec<Yaml>
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ReportError(pub ReportErrorReason);

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ReportErrorReason {
  BadDate,
  ReportParsingError,
  FailedJiraRequest
}

impl Error for ReportError {
  fn description(&self) -> &str {
    match self.0 {
      ReportErrorReason::BadDate => "Was not able to parse a date in the sprint report",
      ReportErrorReason::ReportParsingError => "Error parsing returned JSON",
      ReportErrorReason::FailedJiraRequest => "Using basic auth failed for making a JIRA request"
    }
  }
}

impl fmt::Display for ReportError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.description().fmt(f)
  }
}

fn ordinal(date: NaiveDateTime) -> Result<String, ReportError> {
  let day = date.day();
  let mut result = if day >= 10 {
    String::with_capacity(4)
  } else {
    String::with_capacity(3)
  };
  let shift = if (day / 10) % 10 != 1 && day % 10 < 4 {
    day % 10
  } else {
    0
  };

  result.push_str(&day.to_string());
  match shift {
    0 => result.push_str("th"),
    1 => result.push_str("st"),
    2 => result.push_str("nd"),
    3 => result.push_str("rd"),
    _ => return Err(ReportError(ReportErrorReason::BadDate))
  };
  Ok(result)
}

fn pretty_date(date: NaiveDateTime) -> String {
  let mut result = String::new();
  let day = check!(ordinal(date));
  result += &format!("{} {}, {}", date.format("%A, %B"), day, date.format("%Y"));
  result
}

impl Jira {
  pub fn new(cfg: &Config, users: &str) -> Jira {
    let mut bytes = String::new();
    let mut f = check!(File::open(users));

    if let Ok(b) = f.read_to_string(&mut bytes) {
      info!("Read {} bytes from {}", b, users);
    }
    let doc = &check!(YamlLoader::load_from_str(&bytes))[0];
    let user_vec = check_opt!(doc["users"].to_owned().into_vec());

    Jira {
      cfg: cfg.to_owned(),
      users: user_vec.to_owned()
    }
  }

  fn get_issues(sprint: &JsonValue) -> Result<(Duration, Duration, Duration), ReportError> {
    let completed = sprint["completedIssuesEstimateSum"]["value"].as_fixed_point_i64(0).map(Duration::seconds);
    let incomplete = sprint["incompletedIssuesEstimateSum"]["value"].as_fixed_point_i64(0).map(Duration::seconds);
    let total = sprint["allIssuesEstimateSum"]["value"].as_fixed_point_i64(0).map(Duration::seconds);

    match (completed, incomplete, total) {
      (Some(c), Some(i), Some(t)) => Ok((c, i, t)),
      _ => Err(ReportError(ReportErrorReason::ReportParsingError))
    }
  }

  fn issue_by_user<'a>(d: &'a JsonValue, u: &str) -> Vec<&'a JsonValue> {
    let mut entries: Vec<&'a JsonValue> = Vec::new();
    for j in d.members() {
      if j["assignee"] == u {
        entries.push(j);
      }
    }
    entries
  }

  fn issue_breakdown(&self, name: &str, issues: &JsonValue) -> String {
    let mut result = String::new();
    let base = self.cfg.get_str("jira_base").expect("No base URL in Config");
    let base_url = Url::parse(&base).expect("Can't parse base URL in Config");
    let url = base_url.host_str().expect("No host string available for base URL");
    result += &format!("{} ({})\n", name, issues.len());
    if issues.len() >= 1 {
      for i in issues.members() {
        result += &format!("`{}` ({}) - http://{}/browse/{}\n", i["key"], i["assigneeName"], url, i["key"])
      }
    } else {
      result += &format!("Nothing in {} :(\n", name);
    }
    result
  }

  fn sum_of_issues(d: &[&JsonValue], key: &str) -> i64 {
    let mut total = 0;
    for i in d {
      if i[key]["statFieldValue"].has_key("value") {
        if let Some(v) = i[key]["statFieldValue"]["value"].as_fixed_point_i64(0) {
          total += v
        }
      }
    }
    Duration::seconds(total).num_hours()
  }

  fn get_start_end_dates(sprint: &JsonValue) -> Result<(NaiveDateTime, NaiveDateTime), ReportError> {
    let raw_start = sprint["startDate"].to_string();
    let raw_end = sprint["endDate"].to_string();
    let start = NaiveDateTime::parse_from_str(&raw_start, DATE_FMT);
    let end = NaiveDateTime::parse_from_str(&raw_end, DATE_FMT);
    match (start, end) {
      (Ok(s), Ok(e)) => Ok((s, e)),
      _ => Err(ReportError(ReportErrorReason::BadDate))
    }
  }

  fn jira_request(&self, rapid_id: &str, sprint_id: &str) -> JsonValue {
    let jira_user = self.cfg.get_str("jira_username").expect("No Jira username provided");
    let jira_pw = self.cfg.get_str("jira_password").expect("No Jira password provided");
    let base_url = self.cfg.get_str("jira_base").unwrap();
    let jira_url = format!("{}rest/greenhopper/1.0/rapid/charts/sprintreport", base_url);
    let params = vec![(String::from("rapidViewId"), rapid_id.to_owned()), (String::from("sprintId"), sprint_id.to_owned())];
    let cookies = (jira_user, jira_pw);

    check!(http::http_request(&jira_url, &params, &cookies))
  }

  pub fn sprint_report(&self, rapid_id: &str, sprint_id: &str) -> Result<String, ReportError> {
    let report = self.jira_request(rapid_id, sprint_id);
    Ok(self.build_report(&report))
  }

  pub fn build_report(&self, report: &JsonValue) -> String {
    let mut result = String::new();

    let (start, end) = check!(Jira::get_start_end_dates(&report["sprint"]));
    let now = Local::now();
    let (local_d, local_t) =
      (NaiveDate::from_ymd(now.year(), now.month(), now.day()), NaiveTime::from_hms(now.hour(), now.minute(), now.second()));
    let time_to_end = end.signed_duration_since(NaiveDateTime::new(local_d, local_t));
    let (compl, incompl, total) = Jira::get_issues(&report["contents"]).unwrap();
    let comp_hours = (compl.num_hours() as f64 / total.num_hours() as f64) * 100.0;
    let incomp_hours = (incompl.num_hours() as f64 / total.num_hours() as f64) * 100.0;
    let completed_issues = &report["contents"]["completedIssues"];
    let incompleted_issues = &report["contents"]["incompletedIssues"];

    result += "Report for Sprint: ";
    result += &format!("{}\n", &report["sprint"]["name"]);
    result += &format!("Sprint Start on: {}\n", &pretty_date(start));
    result += &format!("Sprint Ends in: {} day(s) on {}\n", time_to_end.num_days(), &pretty_date(end));
    result += &format!("Completed: {}h ({:.1}%), {} Issues\n", compl.num_hours(), comp_hours, completed_issues.len());
    result += &format!("Incomplete: {}h ({:.1}%), {} Issues\n\n", incompl.num_hours(), incomp_hours, incompleted_issues.len());

    result += "Sprint Sizing:\n";
    for u in &self.users {
      let user = u["name"].to_owned().into_string().unwrap();
      let issues = Jira::issue_by_user(completed_issues, &user);
      let incomplete_issues = Jira::issue_by_user(incompleted_issues, &user);
      let num_issues = issues.len() + incomplete_issues.len();
      if num_issues > 0 {
        let total_comp = Jira::sum_of_issues(&issues, "estimateStatistic");
        let total_incomp = Jira::sum_of_issues(&incomplete_issues, "estimateStatistic");
        let total_hours = (total_comp + total_incomp) as f64;
        let total_pct = (total_comp as f64 / total_hours) * 100.0;
        result +=
          &format!("{} has {} hours left on {} ({:.1}%) hours for {} issues\n", user, total_incomp, total_hours, total_pct, num_issues);
      }
    }
    result += "\n";
    result += &self.issue_breakdown("Completed", completed_issues);
    result += "\n";
    result += &self.issue_breakdown("Incomplete", incompleted_issues);

    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::fs::File;
  use std::io::Read;

  use config::File as CFile;

  use json;

  fn load_data() -> json::JsonValue {
    let mut buf = String::new();
    let mut f = File::open("report.json").unwrap();
    f.read_to_string(&mut buf).unwrap();
    json::parse(&buf).unwrap()
  }

  fn setup() -> Jira {
    let mut cfg = Config::default();
    cfg.merge(CFile::with_name("conf/config.yml"));
    Jira::new(&cfg, "conf/users.yml")
  }

  #[test]
  fn parse_dates() {
    let d = NaiveDate::from_ymd(2018, 1, 3);
    let end_d = NaiveDate::from_ymd(2018, 1, 17);
    let t = NaiveTime::from_hms(10, 48, 0);

    let start = NaiveDateTime::new(d, t);
    let end = NaiveDateTime::new(end_d, t);
    let data = load_data();
    let (s, e) = Jira::get_start_end_dates(&data["sprint"]).unwrap();
    assert_eq!(s, start);
    assert_eq!(e, end);
  }

  #[test]
  fn check_report() {
    let data = load_data();
    let jira = setup();
    println!("{}", jira.build_report(&data));
  }

  #[test]
  fn check_pretty_date() {
    let end_d = NaiveDate::from_ymd(2018, 1, 1);
    let t = NaiveTime::from_hms(0, 0, 0);
    let mut day = NaiveDateTime::new(end_d, t);

    let results = vec![
      "1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th", "10th", "11th", "12th", "13th", "14th", "15th", "16th", "17th",
      "18th", "19th", "20th", "21st", "22nd", "23rd", "24th", "25th", "26th", "27th", "28th", "29th", "30th", "31st",
    ];

    for i in 0..results.len() {
      assert_eq!(results[i].to_string(), ordinal(day).unwrap());
      day += Duration::days(1);
    }
  }
}
