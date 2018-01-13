use slack::{EventHandler, RtmClient, Event, Message};
use slack::api::auth::test;
use slack::api::requests::default_client;

use reqwest::Error;
use jira;

pub struct SlackHandler<'a> {
  token: &'a String,
  my_id: String,
}

impl<'a> SlackHandler<'a> {
  pub fn new<'b>(token: &'b String) -> SlackHandler<'b> {
    SlackHandler {
      token,
      my_id: String::new(),
    }
  }

  pub fn process_message(&self, msg: &String) -> Result<(), Error> {
    if msg.contains(&self.my_id) {
      let tokens: Vec<String> = msg.split_whitespace().map(|s| s.to_owned()).collect();
      let (cmd, rapid_id, sprint_id) = (&tokens[0], &tokens[1], &tokens[2]);
      match cmd.as_ref() {
        "report" => {
          let r = jira::sprint_report(rapid_id, sprint_id);
          println!("Report: {:?}", r);
        }
        _ => {}
      };
    }
    Ok(())
  }
}

#[allow(unused_variables, unused_must_use)]
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
    let response = check!(test(&default_client().unwrap(), &self.token));
    self.my_id = check_opt!(response.user_id);
    println!("My UserId is: {}", self.my_id);
  }
}
