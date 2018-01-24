use slack::{Error, Event, EventHandler, Message, RtmClient};
use slack::api::auth::test;
use slack::api::requests::default_client;

use jira::Jira;

pub struct SlackHandler {
  token: String,
  my_id: String,
  jira: Jira
}

impl SlackHandler {
  pub fn new(token: &str, jira: Jira) -> SlackHandler {
    SlackHandler {
      token: token.to_string(),
      my_id: String::new(),
      jira
    }
  }

  pub fn process_message(&self, msg: &str, channel: &str, cli: &RtmClient) -> Result<(), Error> {
    if msg.contains(&self.my_id) {
      let tokens: Vec<String> = msg.split_whitespace().map(|s| s.to_owned()).collect();
      let cmd = &tokens[1];
      let response: String;
      match cmd.as_ref() {
        "report" => {
          let (rapid_id, sprint_id) = (&tokens[2], &tokens[3]);
          response = check!(self.jira.sprint_report(rapid_id, sprint_id));
        }
        _ => {
          response = format!("I'm not sure what you mean by {}", cmd);
        }
      };
      return match cli.sender().send_message(channel, &response) {
        Ok(_) => Ok(()),
        Err(e) => Err(e)
      }
    }
    Ok(())
  }
}

impl EventHandler for SlackHandler {
  fn on_event(&mut self, cli: &RtmClient, event: Event) {
    if let Event::Message(box Message::Standard(m)) = event {
      if let Some(text) = m.text {
        if let Ok(_) = self.process_message(&text, &m.channel.unwrap(), cli) {}
      }
    }
  }

  fn on_close(&mut self, cli: &RtmClient) {}

  fn on_connect(&mut self, cli: &RtmClient) {
    info!("on_connect");
    let response = check!(test(&default_client().unwrap(), &self.token));
    self.my_id = check_opt!(response.user_id);
    info!("My UserId is: {}", self.my_id);
  }
}
