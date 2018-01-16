use std::io::Read;
use std::io::Error as IOError;

use reqwest::header::{Headers, Cookie};
use reqwest::{Client, Response, Url};

use json::{JsonValue, parse};

pub type Params = Vec<(String, String)>;
pub type Cookies = Vec<(String, String)>;

lazy_static! {
  static ref CLIENT: Client = {
    Client::new()
  };
}

pub fn http_request(url: &str, params: Params, cookies: Cookies) -> JsonValue {
  let mut heads = Headers::new();
  let mut cookie = Cookie::new();
  let base_url = Url::parse_with_params(url, &params).unwrap();

  cookies.iter().for_each(|t| cookie.append(t.0.clone(), t.1.clone()));
  heads.set(cookie);

  let body = CLIENT.get(base_url).headers(heads).send()
    .map(|mut resp: Response| check!(read_body(&mut resp)))
    .map(|s: String| check!(parse(&s)));

  check!(body)
}


fn read_body(response: &mut Response) -> Result<String, IOError> {
  let mut txt = String::new();
  response.read_to_string(&mut txt)?;
  Ok(txt)
}