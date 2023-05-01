use std::env;

use serde::{Deserialize, Serialize};
use tracing::info;

pub async fn meal_init() -> AppState {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
    info!("Initializing...");
    let aws_config = aws_config::load_from_env().await;
    let shared_state = AppState {
        dynamo_client: aws_sdk_dynamodb::Client::new(&aws_config),
        table_name: env::var("TABLE_NAME").expect("DynamoDB table name not set (TABLE_NAME)"),
    };
    shared_state
}

#[derive(Clone)]
pub struct AppState {
    pub dynamo_client: aws_sdk_dynamodb::Client,
    pub table_name: String,
}

#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
    pub message: String,
}

pub fn make_key(id: &str) -> String {
    String::from(format!("MEAL#{}", id))
}
