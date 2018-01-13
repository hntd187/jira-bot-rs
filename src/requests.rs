#[macro_use]
extern crate botlib;

use botlib::jira;

fn main() {
  let data = jira::sprint_report("4978", "14090").unwrap();
  let (start, end) = check!(jira::get_start_end_dates(data["sprint"].clone()));

  println!("Starts: {}\nEnds: {}", start, end);
}