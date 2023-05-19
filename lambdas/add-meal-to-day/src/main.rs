use std::str::FromStr;

use common::{
    aws_sdk_dynamodb::types::AttributeValue,
    common::{make_key, meal_init, AppState},
};
use lambda_http::{
    aws_lambda_events::{chrono::NaiveDate, serde_json::from_str},
    run, service_fn, Body, Error, Request, RequestExt, Response,
};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Meal {
    id: String,
}

/**
 * PATCH /day/2023-05-01
 *
 * { mealId: "24234-235f323-w3-23"}
 */
pub async fn add_meal_to_day(state: &AppState, event: Request) -> Result<Response<Body>, Error> {
    info!("{:#?}", event);
    let meal: Meal =
        from_str(std::str::from_utf8(event.body()).expect("Non utf-8 event")).expect("not json");
    info!("{:#?}", meal);
    let params = event.path_parameters();
    info!("{:#?}", params);
    let day = params.first("day").expect("No day in path!");
    let date: NaiveDate = NaiveDate::from_str(day).expect(&format!("Failed to parse date {}", day));

    let day_key = format!("DAY#{}", date.format("%Y-%m-%d"));
    let meal_key = make_key(&meal.id);

    let request = state
        .dynamo_client
        .put_item()
        .table_name(&state.table_name)
        .item("PK", AttributeValue::S(day_key.to_string()))
        .item("SK", AttributeValue::S(meal_key.to_string()))
        .item(
            "json",
            AttributeValue::S(std::str::from_utf8(event.body()).unwrap().to_owned()),
        );

    request.send().await.expect("PutItem failed");

    let resp = Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body("".into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app_state = meal_init().await;
    run(service_fn(|event: Request| {
        add_meal_to_day(&app_state, event)
    }))
    .await
    .expect("faili");
    Ok(())
}
