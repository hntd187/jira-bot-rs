use std::io::Error as IOError;
use std::io::Read;

use reqwest::{Client, Response, Url};
use reqwest::header::{Cookie, Headers};

use json::{parse, JsonValue};

pub type Params = [(String, String)];
pub type Cookies = [(String, String)];

lazy_static! {
  static ref CLIENT: Client = {
    Client::new()
  };
}

pub fn http_request(url: &str, params: &Params, cookies: &Cookies) -> JsonValue {
  let mut heads = Headers::new();
  let mut cookie = Cookie::new();
  let base_url = check!(Url::parse_with_params(url, params));

  println!("Request sent to {}", base_url);
  cookies
    .iter()
    .for_each(|t| cookie.append(t.0.clone(), t.1.clone()));
  heads.set(cookie);

  let body = CLIENT
    .get(base_url)
    .headers(heads)
    .send()
    .map(|mut resp: Response| check!(read_body(&mut resp)))
    .map(|s: String| check!(parse(&s)));

  check!(body)
}

fn read_body(response: &mut Response) -> Result<String, IOError> {
  let mut txt = String::new();
  response.read_to_string(&mut txt)?;
  Ok(txt)
}