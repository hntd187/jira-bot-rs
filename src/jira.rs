use chrono::prelude::*;
use chrono::ParseError;

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

pub fn get_start_end_dates(sprint: JsonValue) -> Result<(NaiveDateTime, NaiveDateTime), ParseError> {
  let raw_start = sprint["startDate"].to_string();
  let raw_end = sprint["endDate"].to_string();
  let start = NaiveDateTime::parse_from_str(&raw_start, DATE_FMT)?;
  let end = NaiveDateTime::parse_from_str(&raw_end, DATE_FMT)?;
  Ok((start, end))
}

pub fn jira_request(rapid_id: &str, sprint_id: &str) -> JsonValue {
  let session_id = CFG.get_str("session_id").unwrap();
  let base_url = CFG.get_str("jira_base").unwrap();
  let jira_url = format!("{}charts/sprintreport", base_url);
  let params = vec![("rapidViewId".to_string(), rapid_id.to_string()),
                    ("sprintId".to_string(), sprint_id.to_string())];
  let cookies = vec![("JSESSIONID".to_string(), session_id)];

  http::http_request(&jira_url, params, cookies)
}

pub fn sprint_report(rapid_id: &str, sprint_id: &str) -> Result<JsonValue, String> {
  let report = jira_request(rapid_id, sprint_id);
  Ok(report)
}

#[cfg(test)]
mod tests {
  use super::get_start_end_dates;
  use std::fs::File;
  use std::io::Read;

  use json;
  use chrono::prelude::*;

  fn load_data() -> json::JsonValue {
    let mut buf = String::new();
    let mut f = File::open("report.json").unwrap();
    f.read_to_string(&mut buf);
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
    let (s, e) = get_start_end_dates(data["sprint"].clone()).unwrap();
    assert_eq!(s, start);
    assert_eq!(e, end);
  }
}