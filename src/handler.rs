use crate::utils::*;
use actix_web::{get, Responder};
use chrono::Datelike;
use serde_json::{json, Value};
use std::error::Error;

#[get("/api/afdian")]
pub async fn afdian() -> Result<impl Responder, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let first_data = request_afdian_by_page(&client, 1).await?;

    fn parse_list(data: &Value) -> Option<Vec<Value>> {
        let result = data
            .as_object()?
            .get("data")?
            .as_object()?
            .get("list")?
            .as_array()?
            .clone();
        Some(result)
    }
    fn parse_page(data: &Value) -> Option<i32> {
        let total = data
            .as_object()?
            .get("data")?
            .as_object()?
            .get("total_page")?
            .as_i64()?;
        Some(total as i32)
    }
    fn parse_data(list: &Vec<Value>) -> Option<Vec<Value>> {
        let mut result: Vec<Value> = Vec::new();

        for item in list {
            let date = item.as_object()?.get("last_pay_time")?.as_i64()?;
            let date_time = chrono::NaiveDateTime::from_timestamp_millis(date * 1000)?;
            let date = format!(
                "{}-{}-{}",
                date_time.year(),
                date_time.month(),
                date_time.day()
            );
            let avatar = item
                .as_object()?
                .get("user")?
                .as_object()?
                .get("avatar")?
                .as_str()?;
            let name = item
                .as_object()?
                .get("user")?
                .as_object()?
                .get("name")?
                .as_str()?;
            let user_id = item
                .as_object()?
                .get("user")?
                .as_object()?
                .get("user_id")?
                .as_str()?;
            let money = item.as_object()?.get("all_sum_amount")?.as_str()?;
            result.push(json!({
                "date": date,
                "avatar": avatar,
                "name":name,
                "user_id":user_id,
                "money": money
            }))
        }
        Some(result)
    }
    let mut list: Vec<Value> = Vec::new();
    if let Some(data) = parse_list(&first_data) {
        list = vec![list, data].concat();
    } else {
        return Ok("parse_list error".to_string());
    }
    let total_page = parse_page(&first_data).unwrap_or(1);
    if total_page > 1 {
        for page in 2..=total_page {
            let page_data = request_afdian_by_page(&client, page as i32).await?;
            if let Some(data) = parse_list(&page_data) {
                list = vec![list, data].concat();
            } else {
                return Ok("parse_list error".to_string());
            }
        }
    }

    let parsed_data = parse_data(&list).unwrap_or(Vec::new());

    Ok(Value::Array(parsed_data).to_string())
}
