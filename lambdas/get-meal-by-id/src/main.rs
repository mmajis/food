use common::{
    aws_sdk_dynamodb::types::AttributeValue,
    common::{make_key, meal_init, AppState},
    serde_dynamo::aws_sdk_dynamodb_0_26::from_item,
};
use lambda_http::{
    aws_lambda_events::serde_json::{json, Value},
    run, service_fn, Body, Error, Request, RequestExt, Response,
};
use tracing::info;

async fn get_meal(state: &AppState, event: Request) -> Result<Response<Body>, Error> {
    info!("{:#?}", event);
    let params = event.path_parameters();
    info!("{:#?}", params);
    let id = params.first("meal_id").expect("No meal_id!");
    let key = make_key(id);

    let request = state
        .dynamo_client
        .get_item()
        .table_name(&state.table_name)
        .key("PK", AttributeValue::S(key.to_string()))
        .key("SK", AttributeValue::S(key.to_string()));
    let response = request.send().await.expect("DynamoDB query failed");

    let item = response.item().unwrap();
    let jaha: Value = from_item(item.to_owned()).unwrap();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(jaha["json"].as_str().unwrap().into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app_state = meal_init().await;
    run(service_fn(|event: Request| get_meal(&app_state, event)))
        .await
        .expect("faili");
    Ok(())
}
