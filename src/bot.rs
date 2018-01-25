use slack::{Error, Event, EventHandler, Message, RtmClient};

use jira::Jira;

pub struct SlackHandler {
  my_id: String,
  jira: Jira
}

impl SlackHandler {
  pub fn new(jira: Jira) -> SlackHandler {
    SlackHandler {
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
      }
      cli.sender().send_message(channel, &response)?;
    }
    Ok(())
  }
}

impl EventHandler for SlackHandler {
  fn on_event(&mut self, cli: &RtmClient, event: Event) {
    if let Event::Message(box Message::Standard(m)) = event {
      if let Some(text) = m.text {
        check!(self.process_message(&text, &m.channel.unwrap(), cli))
      }
    }
  }

  #[allow(unused_variables)]
  fn on_close(&mut self, cli: &RtmClient) {}

  #[allow(unused_variables)]
  fn on_connect(&mut self, cli: &RtmClient) {
    let me = cli.start_response().clone().slf.unwrap();
    self.my_id = check_opt!(me.id);
    if let Some(n) = me.name {
      info!("Connected as {:?}", n);
    }
    info!("My User ID is: {}", self.my_id);
  }
}
