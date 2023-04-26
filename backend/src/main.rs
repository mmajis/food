mod api;
use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use lambda_web::{is_running_on_lambda, run_hyper_on_lambda, LambdaError};
use serde_json::{json, Value};
use std::{env, net::SocketAddr};
use tracing::{debug, error, info, span, warn, Level};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    dynamo_client: aws_sdk_dynamodb::Client,
    table_name: String,
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .json()
        .init();
    // initialize the app state
    info!("Initializing...");
    let aws_config = aws_config::load_from_env().await;
    let shared_state = AppState {
        dynamo_client: aws_sdk_dynamodb::Client::new(&aws_config),
        table_name: env::var("TABLE_NAME").expect("DynamoDB table name not set (TABLE_NAME)"),
    };

    // build our application with a route
    let app = Router::new()
        .route("/", get(root))
        .route("/meal", post(api::store_meal))
        .route("/meal/:meal_id", get(api::get_meal))
        .with_state(shared_state);

    if is_running_on_lambda() {
        // Run app on AWS Lambda
        run_hyper_on_lambda(app).await?;
    } else {
        // Run app on local server
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .await?;
    }
    Ok(())
}
