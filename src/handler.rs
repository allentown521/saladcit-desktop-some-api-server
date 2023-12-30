use crate::utils::*;
use actix_web::{get, post, web, Responder};
use chrono::Datelike;
use futures::TryStreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::{Connection, Row, SqliteConnection};
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

#[derive(Deserialize)]
pub struct Info {
    text: String,
}

#[post("/api/dict")]
pub async fn dict(info: web::Json<Info>) -> Result<impl Responder, Box<dyn Error>> {
    let mut conn = SqliteConnection::connect(&format!("sqlite:stardict.db")).await?;

    let mut rows = sqlx::query("SELECT * FROM stardict WHERE word = ?")
        .bind(&info.text)
        .fetch(&mut conn);

    while let Some(row) = rows.try_next().await? {
        let phonetic: String = row.try_get("phonetic")?;
        let translation: String = row.try_get("translation")?;
        let tag: String = row.try_get("tag")?;
        let exchange: String = row.try_get("exchange")?;
        let translation_list = translation.split("\n").collect::<Vec<&str>>();
        let mut explanations: Vec<Value> = Vec::new();
        let mut associations: Vec<String> = Vec::new();

        for line in translation_list {
            let temp = line.split(".").collect::<Vec<&str>>();
            let mut trait_name = "";
            let mut explains = Vec::new();

            if temp.len() > 1 {
                trait_name = temp[0];
                explains = temp[1].split(",").collect::<Vec<&str>>();
            } else {
                trait_name = "";
                explains = temp[0].split(",").collect::<Vec<&str>>();
            }
            let mut explain_list: Vec<Value> = Vec::new();
            for explain in explains {
                explain_list.push(Value::String(explain.to_string()));
            }
            explanations.push(json!({
                "trait": trait_name,
                "explains": explain_list
            }));
        }
        if !exchange.is_empty() {
            for item in exchange.split("/") {
                let temp = item.split(":").collect::<Vec<&str>>();

                let word = temp[1];
                match temp[0] {
                    "p" => associations.push(format!("过去式: {word}")),
                    "d" => associations.push(format!("过去分词: {word}")),
                    "i" => associations.push(format!("现在分词: {word}")),
                    "3" => associations.push(format!("第三人称单数: {word}")),
                    "r" => associations.push(format!("比较级: {word}")),
                    "t" => associations.push(format!("最高级: {word}")),
                    "s" => associations.push(format!("复数: {word}")),
                    "0" => associations.push(format!("Lemma: {word}")),
                    "1" => associations.push(format!("Lemma: {word}")),
                    _ => {}
                }
            }
        }

        if !tag.is_empty() {
            associations.push("".to_string());
            associations.push(tag);
        }
        let mut result = json!({
          "explanations": explanations
        });
        if !phonetic.is_empty() {
            result.as_object_mut().unwrap().insert(
                "pronunciations".to_string(),
                json!([
                  {
                    "symbol": format!("/{phonetic}/")
                  }
                ]),
            );
        }
        if !associations.is_empty() {
            result
                .as_object_mut()
                .unwrap()
                .insert("associations".to_string(), associations.into());
        }

        return Ok(result.to_string());
    }
    Err("Not found".into())
}

#[post("/api/ali_qrcode")]
pub async fn ali_qrcode() -> Result<impl Responder, Box<dyn Error>> {
    let client = reqwest::Client::new();

    let res: Value = client
        .post("https://openapi.alipan.com/oauth/authorize/qrcode")
        .header("Content-Type", "application/json")
        .json(&json!({
          "client_id": crate::ALIPAN_CLIENTID,
          "client_secret":crate::ALIPAN_SECRET,
          "scopes": [
            "user:base",
            "file:all:read",
            "file:all:write"
          ]
        }))
        .send()
        .await?
        .json()
        .await?;
    Ok(res.to_string())
}

#[derive(Deserialize)]
pub struct Code {
    code: String,
    refresh_token: String,
}

#[post("/api/ali_access_token")]
pub async fn ali_access_token(code: web::Json<Code>) -> Result<impl Responder, Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut body = json!({
      "client_id": crate::ALIPAN_CLIENTID,
      "client_secret":crate::ALIPAN_SECRET,
      "grant_type": "authorization_code",
    });
    if !code.code.is_empty() {
        body.as_object_mut()
            .unwrap()
            .insert("code".to_string(), json!(code.refresh_token));
    }
    if !code.refresh_token.is_empty() {
        body.as_object_mut()
            .unwrap()
            .insert("refresh_token".to_string(), json!(code.refresh_token));
    }
    let res: Value = client
        .post("https://openapi.alipan.com/oauth/access_token")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?
        .json()
        .await?;
    Ok(res.to_string())
}
