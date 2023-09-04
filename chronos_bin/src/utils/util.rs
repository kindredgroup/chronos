use rdkafka::message::{BorrowedHeaders, BorrowedMessage, Header, Headers, OwnedHeaders};
use rdkafka::Message;
use std::collections::HashMap;

pub static CHRONOS_ID: &str = "chronosId";
pub static DEADLINE: &str = "chronosDeadline";

//TODO check correctness for two headers in this method
pub fn required_headers(message: &BorrowedMessage) -> Option<HashMap<String, String>> {
    if let Some(headers) = message.headers() {
        let reqd_headers = headers.iter().fold(HashMap::<String, String>::new(), |mut acc, header| {
            let key: String = header.key.parse().unwrap();
            let value: String = String::from_utf8_lossy(header.value.expect("utf8 parsing for header value failed")).into_owned();

            acc.insert(key, value);
            acc
        });
        return Some(reqd_headers);
    }
    None
}

pub fn into_headers(headers: &HashMap<String, String>) -> OwnedHeaders {
    headers.iter().fold(OwnedHeaders::new(), |acc, header| {
        let (key, value) = header;
        let o_header = Header { key, value: Some(value) };
        acc.insert(o_header)
    })
}

pub fn headers_check(headers: &BorrowedHeaders) -> bool {
    let outcome = headers
        .iter()
        .filter(|h| {
            let header_keys = [CHRONOS_ID, DEADLINE];
            header_keys.contains(&h.key) && h.value.is_some()
        })
        .count()
        == 2;

    outcome
}

pub fn get_payload_utf8<'a>(message: &'a BorrowedMessage) -> &'a [u8] {
    message.payload().expect("parsing payload failed")
}

pub fn get_message_key(message: &BorrowedMessage) -> String {
    let key = String::from_utf8_lossy(message.key().expect("No key found for message")).to_string();
    key
}
