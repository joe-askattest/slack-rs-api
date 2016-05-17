// Copyright 2015-2016 the slack-rs authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! For more information, see [Slack's API
//! documentation](https://api.slack.com/methods).

use std::collections::HashMap;
use std::io::Read;
use hyper;
use rustc_serialize::json;

use super::ApiResult;
use error::Error;

/// Exchanges a temporary OAuth code for an API token.
///
/// Wraps https://api.slack.com/methods/oauth.access
pub fn access(client: &hyper::Client, client_id: &str, client_secret: &str, code: &str, redirect_uri: Option<&str>) -> ApiResult<AccessResponse> {
    let mut params = HashMap::new();
    params.insert("client_id", client_id);
    params.insert("client_secret", client_secret);
    params.insert("code", code);
    if let Some(redirect_uri) = redirect_uri {
        params.insert("redirect_uri", redirect_uri);
    }

    let mut url = hyper::Url::parse("https://slack.com/api/oauth.access").expect("Unable to parse url");
    url.query_pairs_mut().extend_pairs(params.into_iter());

    let response = try!(client.get(url).send());
    transform_oauth_response(response)
}

fn transform_oauth_response(mut res: hyper::client::response::Response) -> ApiResult<AccessResponse> {
    let mut res_str = String::new();
    try!(res.read_to_string(&mut res_str));

    let raw_json = try!(json::Json::from_str(&res_str));
    let jobj = try!(raw_json.as_object()
                            .ok_or(Error::Api(format!("bad slack json response (not an object) {:?}", raw_json))));
    if let Some(ok) = jobj.get("ok") {
        let is_ok = try!(ok.as_boolean()
                           .ok_or(Error::Api(format!("slack json reponse \"ok\" is not a boolean: {:?}", raw_json))));
        if !is_ok {
            return Err(Error::Api(format!("slack json reponse \"ok\" is not true: {:?}", raw_json)));
        }
    }

    Ok(try!(json::decode(&res_str)))
}

#[derive(Clone,Debug,RustcDecodable)]
pub struct AccessResponse {
    pub access_token: String,
    pub scope: String,
}

#[cfg(test)]
mod tests {
    use hyper;
    use super::*;

    mock_slack_responder!(MockErrorResponder, r#"{"ok": false, "err": "some_error"}"#);

    #[test]
    fn general_api_error_response() {
        let client = hyper::Client::with_connector(MockErrorResponder::default());
        let result = access(&client, "TEST_ID", "TEST_TOKEN", "TEST_CODE", None);
        assert!(result.is_err());
    }

    mock_slack_responder!(MockListOkResponder,
        r#"{
            "access_token": "xoxt-23984754863-2348975623103",
            "scope": "read"
        }"#
    );

    #[test]
    fn access_ok_response() {
        let client = hyper::Client::with_connector(MockListOkResponder::default());
        let result = access(&client, "TEST_ID", "TEST_TOKEN", "TEST_CODE", None);
        if let Err(err) = result {
            panic!(format!("{:?}", err));
        }
        assert_eq!(result.unwrap().access_token,
                   "xoxt-23984754863-2348975623103");
    }
}
