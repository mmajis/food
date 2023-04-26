use std::collections::HashMap;

use crate::AppState;
use aws_sdk_dynamodb::types::AttributeValue;
use aws_sdk_dynamodb::{Client, Error};
use axum::extract::{Path, State};
use axum::response::Json;
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};
use serde_dynamo::aws_sdk_dynamodb_0_26::from_item;
use serde_json::{json, to_value, Value};
use tracing::log::info;
use uuid::Uuid;

//#[debug_handler]
pub async fn store_meal(State(state): State<AppState>, Json(payload): Json<Value>) -> () {
    println!("{:?}", payload);
    info!("eah");
    let key = get_key(&payload);

    let request = state
        .dynamo_client
        .put_item()
        .table_name(state.table_name)
        .item("PK", AttributeValue::S(key.to_string()))
        .item("SK", AttributeValue::S(key.to_string()))
        .item(
            "name",
            AttributeValue::S(payload.get("name").unwrap().to_string()),
        )
        .item("json", AttributeValue::S(payload.to_string()));

    request.send().await.expect("PutItem failed");
}

//#[debug_handler]
pub async fn get_meal(State(state): State<AppState>, Path(id): Path<Uuid>) -> Json<Value> {
    let key = make_key(id.to_string().as_str());

    let request = state
        .dynamo_client
        .get_item()
        .table_name(state.table_name)
        .key("PK", AttributeValue::S(key.to_string()))
        .key("SK", AttributeValue::S(key.to_string()));
    let response = request.send().await.expect("DynamoDB query failed");

    let item = response.item().unwrap();
    let jaha: Value = from_item(item.to_owned()).unwrap();
    Json(jaha)
}

fn get_key(meal: &Value) -> String {
    let uuid = json!(Uuid::new_v4().to_string());
    let id = meal.get("id").unwrap_or(&uuid);
    make_key(id.to_string().as_str())
}

fn make_key(id: &str) -> String {
    String::from(format!("MEAL#{}", id))
}
