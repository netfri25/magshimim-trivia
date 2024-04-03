use serde::Deserialize;
use serde_repr::{Deserialize_repr, Serialize_repr};

use base64::engine::general_purpose::STANDARD as base64;
use base64::Engine as _;

use super::question::Question;

pub fn get_questions(amount: u8) -> anyhow::Result<Vec<Question>> {
    let url = format!(
        "https://opentdb.com/api.php?amount={}&encode=base64",
        amount
    );
    let resp_string = ureq::get(&url).call()?.into_string()?;
    let mut resp_json: serde_json::Value = serde_json::from_str(&resp_string)?;
    decode_json_base64(&mut resp_json)?;

    let resp: Response = serde_json::from_value(resp_json)?;

    if resp.response_code != ResponseCode::Success {
        return Err(resp.response_code.into());
    }

    let questions = resp.results;
    Ok(questions)
}

fn decode_json_base64(value: &mut serde_json::Value) -> anyhow::Result<()> {
    match value {
        serde_json::Value::String(s) => {
            let decoded = base64.decode(s.as_bytes())?;
            *s = String::from_utf8(decoded)?;
        }

        serde_json::Value::Array(arr) => {
            arr.iter_mut().try_for_each(decode_json_base64)?;
        }

        serde_json::Value::Object(obj) => {
            obj.values_mut().try_for_each(decode_json_base64)?;
        }

        _ => {}
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Response {
    response_code: ResponseCode,
    results: Vec<Question>,
}

#[derive(thiserror::Error, Debug, Serialize_repr, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum ResponseCode {
    #[error("none")]
    Success = 0,

    #[error("no results")]
    NoResults = 1,

    #[error("invalid parameter")]
    InvalidParameter = 2,

    #[error("token not found")]
    TokenNotFound = 3,

    #[error("token empty")]
    TokenEmpty = 4,

    #[error("rate limit")]
    RateLimit = 5,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "prevent API spamming"]
    pub fn correct_parsing() {
        get_questions(2).unwrap();
    }
}
