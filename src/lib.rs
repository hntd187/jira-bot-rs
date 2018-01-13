#![feature(box_patterns, try_trait)]
extern crate slack;
extern crate config;
extern crate reqwest;
extern crate json;
extern crate chrono;

#[macro_use]
extern crate lazy_static;

#[macro_export]
macro_rules! check {
    ($expr:expr) => (match $expr {
        Ok(val)  => val,
        Err(err) => panic!("Error: {}", err)
    });
}

#[macro_export]
macro_rules! check_opt {
  ($expr:expr) => (match $expr {
        Some(val)  => val,
        None => panic!("Encountered None for {:?}", $expr)
  });
}

pub mod bot;
pub mod jira;
mod http;

