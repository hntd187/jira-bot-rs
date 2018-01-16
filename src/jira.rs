use std::fmt;
use std::fmt::Write;
use std::error::Error;

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
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ReportError(ReportErrorReason);

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum ReportErrorReason {
  BadDate,
  BadRequest,
}

impl Error for ReportError {
  fn description(&self) -> &str {
    match self.0 {
      ReportErrorReason::BadDate => "Was not able to parse a date in the sprint report",
      ReportErrorReason::BadRequest => "The REST called to jira failed"
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

fn get_issues(sprint: &JsonValue) -> Result<(Duration, Duration, Duration), ReportError> {
  let completed = sprint["completedIssuesEstimateSum"]["value"].as_i64().map(Duration::seconds);
  let incomplete = sprint["incompletedIssuesEstimateSum"]["value"].as_i64().map(Duration::seconds);
  let total = sprint["allIssuesEstimateSum"]["value"].as_i64().map(Duration::seconds);

  Ok((completed.unwrap(), incomplete.unwrap(), total.unwrap()))
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
  let params = vec![("rapidViewId".to_string(), rapid_id.to_string()),
                    ("sprintId".to_string(), sprint_id.to_string())];
  let cookies = vec![("JSESSIONID".to_string(), session_id)];

  http::http_request(&jira_url, params, cookies)
}

pub fn sprint_report(rapid_id: &str, sprint_id: &str) -> Result<String, ReportError> {
  let report = jira_request(rapid_id, sprint_id);
  let mut result = String::new();

  let (start, end) = check!(get_start_end_dates( & report["sprint"]));
  let time_to_end = end.signed_duration_since(start);

  result += &format!("Report for Sprint: {}\n", report["sprint"]["name"]);
  result += &format!("Sprint Ends in: {} on {}\n", time_to_end, end);
  Ok(result)
}

#[allow(unused_must_use)]
pub fn build_report(report: JsonValue) -> String {
  let mut result = String::new();

  let (start, end) = check!(get_start_end_dates( & report["sprint"]));
  let time_to_end: Duration = end.signed_duration_since(start);
  let (compl, incompl, total) = get_issues(&report["contents"]).unwrap();

  result.push_str("Report for Sprint: ");
  result.push_str(format!("{}\n", &report["sprint"]["name"]));
  result.push_str(format!("Sprint Ends in: {} day(s) on {}\n", time_to_end.num_days(), end));

  result
}

#[cfg(test)]
mod tests {
  use super::*;

  use std::fs::File;
  use std::io::Read;

  use json;
  use chrono::*;

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