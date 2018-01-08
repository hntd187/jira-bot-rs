extern crate slack;
extern crate slack_api;
extern crate config;

use slack::EventHandler;
use slack::RtmClient;
use slack::Event;
use slack::Message;
use slack::api::auth::test;
use slack::api::requests::Client;

use config::Config;
use config::File;

const COMMANDS: [&'static str; 3] = ["report", "burndown", "help"];

struct SlackHandler<'a> {
    token: &'a String,
    client: Client,
    my_id: String,
}

#[allow(unused_variables)]
impl<'a> SlackHandler<'a> {
    pub fn new(token: &String) -> SlackHandler {
        SlackHandler {
            token,
            client: slack_api::default_client().unwrap(),
            my_id: String::from(""),
        }
    }
}

#[allow(unused_variables)]
impl<'a> EventHandler for SlackHandler<'a> {
    fn on_event(&mut self, cli: &RtmClient, event: Event) {
        match event {
            Event::Message(bm) => {
                match *bm {
                    Message::Standard(msg) => {},
                    _ => {}
                }
            }
            _ => {}
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

fn main() {
    let mut cfg = Config::default();
    cfg.merge(File::with_name("conf/config.yml"));

    let token = cfg.get_str("slack_key").unwrap();
    let mut handler = SlackHandler::new(&token);
    let r = RtmClient::login_and_run(&token, &mut handler);
    match r {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
