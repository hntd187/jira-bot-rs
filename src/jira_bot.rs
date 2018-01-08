extern crate slack;
extern crate slack_api;
extern crate config;
extern crate reqwest;

use slack::{EventHandler, RtmClient, Event, Message};
use slack::api::auth::test;
use slack::api::requests::Client;
use slack::api::default_client;
use reqwest::Url;
use reqwest::Error;

pub struct SlackHandler<'a> {
  token: &'a String,
  base: &'a String,
  client: Client,
  my_id: String,
}

impl<'a> SlackHandler<'a> {
  pub fn new<'b>(token: &'b String, base: &'b String) -> SlackHandler<'b> {
    SlackHandler {
      token,
      base,
      client: default_client().unwrap(),
      my_id: String::from(""),
    }
  }

  pub fn process_message(&self, msg: &String) -> Result<(), Error> {
    if msg.contains(&self.my_id) {
      let tokens: Vec<String> = msg.split_whitespace().map(|s| s.to_owned()).collect();
      let (cmd, rapid_id, sprint_id) = (&tokens[0], &tokens[1], &tokens[2]);
      match cmd.as_ref() {
        "report" => {
          let r = self.sprint_report(rapid_id, sprint_id)?;
          println!("Report: {}", r);
        }
        _ => {}
      };
    }
    Ok(())
  }

  fn sprint_report(&self, rapid_id: &String, sprint_id: &String) -> Result<&str, Error> {
    let params = [("rapidId", rapid_id), ("sprintId", sprint_id)];

    let base_url = match Url::parse_with_params(self.base.as_str(), &params) {
      Ok(u) => u,
      Err(e) => panic!("Error parsing url: {:?} with {:?}, Reason: {:?}", self.base, params.to_vec(), e),
    };
    let joined_url = base_url.join("charts/sprintreport").unwrap();
    let resp = self.client.get(joined_url.as_str());

    return Ok("Body!");
  }
}

#[allow(unused_variables)]
impl<'a, 'b> EventHandler for SlackHandler<'a> {
  fn on_event(&mut self, cli: &RtmClient, event: Event) {
    if let Event::Message(box Message::Standard(m)) = event {
      if let Some(text) = m.text {
        self.process_message(&text);
      }
    }
  }


  fn on_close(&mut self, cli: &RtmClient) {
    println!("on_close");
  }

  fn on_connect(&mut self, cli: &RtmClient) {
    println!("on_connect");
    self.my_id = match test(&self.client, &self.token) {
      Ok(tr) => tr.user_id.unwrap(),
      Err(err) => panic!("Error in user ID: {}", err)
    };
    println!("My UserId is: {}", self.my_id);
  }
}
