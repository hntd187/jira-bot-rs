extern crate json;

use std::fmt;
use std::io::Read;
use std::error::Error;
use std::fs::File as StdFile;

use chrono::*;
use config::*;
use json::JsonValue;

use http;

const DATE_FMT: &'static str = "%d/%b/%y %I:%M %p";

lazy_static! {
  static ref CFG: Config = {
    let mut c = Config::default();
    c.merge(File::with_name("conf/config.yml"));
    c
  };
  static ref USERS: Vec<String> = {
      let mut data = String::new();
      let mut file = StdFile::open("conf/users.json").unwrap();
      file.read_to_string(&mut data).expect("Failed to read users.json...");
      json::parse(&data)
        .expect("Failed to parse users.json file")["users"]
        .take()
        .members()
        .map(|s| s.to_string())
        .collect()
  };
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ReportError(ReportErrorReason);

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum ReportErrorReason {
  BadDate,
  BadRequest,
  ReportParsingError,
}

impl Error for ReportError {
  fn description(&self) -> &str {
    match self.0 {
      ReportErrorReason::BadDate => "Was not able to parse a date in the sprint report",
      ReportErrorReason::BadRequest => "The REST called to jira failed",
      ReportErrorReason::ReportParsingError => "Error parsing returned JSON"
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
  let mut result = if day >= 10 { String::with_capacity(4) } else { String::with_capacity(3) };
  let shift = if (day as f32 / 10.0).floor() % 10.0 != 1.0 && day % 10 < 4 { day % 10 } else { 0 };

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
  result.push_str(&format!("{} {}, {}", date.format("%A, %B"), day, date.format("%Y")));
  result
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

fn issue_by_user<'a>(d: &'a JsonValue, u: &'a str) -> Vec<&'a JsonValue> {
  let mut entries: Vec<&'a JsonValue> = Vec::new();
  for j in d.members() {
    if j["assignee"] == u {
      entries.push(&j);
    }
  }
  entries
}

fn issue_breakdown(name: &str, issues: &JsonValue) -> String {
  let mut result = String::new();
  let base_url = CFG.get_str("jira_base").expect("No JIRA Base URL in Config");
  result += &format!("{} ({})\n", name, issues.len());
  if issues.len() >= 1 {
    for i in issues.members() {
      result += &format!("`{}` ({}) - {}browse/{}\n", i["key"], i["assigneeName"], base_url, i["key"])
    }
  } else {
    result += &format!("Nothing in {} :(\n", name);
  }
  result
}

fn sum_of_issues<'a>(d: Vec<&JsonValue>, key: &'a str) -> i64 {
  let mut total = 0;
  for i in d.iter() {
    if i[key]["statFieldValue"].has_key("value") {
      match i[key]["statFieldValue"]["value"].as_fixed_point_i64(0) {
        Some(v) => total += v,
        None => {}
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

fn jira_request(rapid_id: &str, sprint_id: &str) -> JsonValue {
  let session_id = CFG.get_str("session_id").unwrap();
  let base_url = CFG.get_str("jira_base").unwrap();
  let jira_url = format!("{}charts/sprintreport", base_url);
  let params = vec![
    (String::from("rapidViewId"), rapid_id.to_owned()),
    (String::from("sprintId"), sprint_id.to_owned())
  ];
  let cookies = vec![(String::from("JSESSIONID"), session_id.to_owned())];

  http::http_request(&jira_url, params, cookies)
}

pub fn sprint_report(rapid_id: &str, sprint_id: &str) -> Result<String, ReportError> {
  let report = jira_request(rapid_id, sprint_id);
  Ok(build_report(report))
}

#[allow(unused_must_use)]
pub fn build_report(report: JsonValue) -> String {
  let mut result = String::new();

  let (start, end) = check!(get_start_end_dates(&report["sprint"]));
  let time_to_end = end.signed_duration_since(start);
  let (compl, incompl, total) = get_issues(&report["contents"]).unwrap();
  let comp_hours = (compl.num_hours() as f64 / total.num_hours() as f64) * 100.0;
  let incomp_hours = (incompl.num_hours() as f64 / total.num_hours() as f64) * 100.0;
  let completed_issues = &report["contents"]["completedIssues"];
  let incompleted_issues = &report["contents"]["incompletedIssues"];

  result += "Report for Sprint: ";
  result += &format!("{}\n", &report["sprint"]["name"]);
  result += &format!("Sprint Ends in: {} day(s) on {}\n", time_to_end.num_days(), &pretty_date(end));
  result += &format!("Completed: {}h ({:.1}%), {} Issues\n", compl.num_hours(), comp_hours, completed_issues.len());
  result += &format!("Incomplete: {}h ({:.1}%), {} Issues\n\n", incompl.num_hours(), incomp_hours, incompleted_issues.len());

  result += "Sprint Sizing:\n";
  for u in USERS.iter() {
    let issues = issue_by_user(completed_issues, &u);
    let incomplete_issues = issue_by_user(incompleted_issues, &u);
    let num_issues = issues.len() + incomplete_issues.len();
    if num_issues > 0 {
      let total_comp = sum_of_issues(issues, "estimateStatistic");
      let total_incomp = sum_of_issues(incomplete_issues, "estimateStatistic");
      let total_hours = (total_comp + total_incomp) as f64;
      let total_pct = (total_comp as f64 / total_hours) * 100.0;
      result += &format!("{} has {} hours left on {} ({:.1}%) hours for {} issues\n", u, total_incomp, total_hours, total_pct, num_issues);
    }
  }
  result += "\n";
  result += &issue_breakdown("Completed", completed_issues);
  result += "\n";
  result += &issue_breakdown("Incomplete", incompleted_issues);

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::fs::File;
  use std::io::Read;

  use json;

  fn load_data() -> json::JsonValue {
    let mut buf = String::new();
    let mut f = File::open("report.json").unwrap();
    f.read_to_string(&mut buf).unwrap();
    json::parse(&buf).unwrap()
  }

  #[test]
  fn parse_dates() {
    let d = NaiveDate::from_ymd(2018, 1, 3);
    let end_d = NaiveDate::from_ymd(2018, 1, 17);
    let t = NaiveTime::from_hms(10, 48, 0);

    let start = NaiveDateTime::new(d, t);
    let end = NaiveDateTime::new(end_d, t);
    let data = load_data();
    let (s, e) = get_start_end_dates(&data["sprint"]).unwrap();
    assert_eq!(s, start);
    assert_eq!(e, end);
  }

  #[test]
  fn check_report() {
    let data = load_data();
    println!("{}", build_report(data));
  }

  #[test]
  fn check_pretty_date() {
    let end_d = NaiveDate::from_ymd(2018, 1, 1);
    let t = NaiveTime::from_hms(0, 0, 0);
    let mut day = NaiveDateTime::new(end_d, t);

    let results = vec!["1st", "2nd", "3rd", "4th", "5th", "6th", "7th", "8th", "9th", "10th",
                       "11th", "12th", "13th", "14th", "15th", "16th", "17th", "18th", "19th",
                       "20th", "21st", "22nd", "23rd", "24th", "25th", "26th", "27th", "28th",
                       "29th", "30th", "31st"];

    for i in 0..results.len() {
      assert_eq!(results[i].to_string(), ordinal(day).unwrap());
      day += Duration::days(1);
    }
  }
}
