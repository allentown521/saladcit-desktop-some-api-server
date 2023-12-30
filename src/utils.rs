use crate::{AFDIAN_TOKEN, AFDIAN_USER_ID};
use chrono::prelude::*;
use reqwest::Client;
use serde_json::{json, Value};

pub async fn request_afdian_by_page(
    client: &Client,
    page: i32,
) -> Result<Value, Box<dyn std::error::Error>> {
    let user_id = AFDIAN_USER_ID;
    let token = AFDIAN_TOKEN;
    let params = json!({ "page": page }).to_string();
    let ts = Utc::now().timestamp();
    let sign = md5::compute(format!("{token}params{params}ts{ts}user_id{user_id}"));

    let res: Value = client
        .post("https://afdian.net/api/open/query-sponsor")
        .header("Content-Type", "application/json")
        .json(&json!({
            "user_id":user_id,
            "params":params,
            "ts":ts,
            "sign":format!("{:x}", sign)
        }))
        .send()
        .await?
        .json()
        .await?;
    Ok(res)
}
