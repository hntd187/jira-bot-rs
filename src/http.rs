use std::io::Error as IOError;
use std::io::Read;

use reqwest::{Client, Response, Url};
use reqwest::header::{Authorization, Basic, Headers};

use json::{parse, JsonValue};

use jira::{ReportError, ReportErrorReason};

pub type Params = [(String, String)];
pub type Auth = (String, String);

lazy_static! {
  static ref CLIENT: Client = {
    Client::new()
  };
}

pub fn http_request(url: &str, params: &Params, auth: &Auth) -> Result<JsonValue, ReportError> {
  let mut heads = Headers::new();
  let auth_header = Authorization(Basic {
    username: auth.0.to_owned(),
    password: Some(auth.1.to_owned())
  });
  let base_url = check!(Url::parse_with_params(url, params));
  heads.set(auth_header);

  info!("Request sent to {}", base_url);
  let request = match CLIENT.get(base_url).headers(heads).send() {
    Ok(r) => Ok(r),
    Err(_) => Err(ReportError(ReportErrorReason::FailedJiraRequest))
  }.and_then(|r| {
    if r.status().is_success() {
      Ok(r)
    } else {
      Err(ReportError(ReportErrorReason::FailedJiraRequest))
    }
  });

  let response = request.map(|mut resp| check!(read_body(&mut resp))).map(|s: String| check!(parse(&s)));

  Ok(check!(response))
}

fn read_body(response: &mut Response) -> Result<String, IOError> {
  let mut txt = String::new();
  match response.read_to_string(&mut txt) {
    Ok(b) => {
      info!("Read {} bytes from response: {}", b, response.url());
      Ok(txt)
    }
    Err(e) => Err(e)
  }
}
