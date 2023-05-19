use common::{
    aws_sdk_dynamodb::types::AttributeValue,
    common::{make_key, meal_init, AppState},
};
use lambda_http::{
    aws_lambda_events::serde_json::{from_str, json, Value},
    run, service_fn, Body, Error, Request, Response,
};
use tracing::info;
use uuid::Uuid;

pub async fn store_meal(state: &AppState, event: Request) -> Result<Response<Body>, Error> {
    println!("{:#?}", event);
    let mut payload: Value =
        from_str(std::str::from_utf8(event.body()).expect("Non utf-8 event")).expect("not json");
    let key = get_key(&payload);

    payload["id"] = json!(key[5..]);

    info!("{:#?}", payload);

    let request = state
        .dynamo_client
        .put_item()
        .table_name(&state.table_name)
        .item("PK", AttributeValue::S(key.to_string()))
        .item("SK", AttributeValue::S(key.to_string()))
        .item(
            "name",
            AttributeValue::S(
                payload
                    .get("name")
                    .expect("Attribute \"name\" is missing")
                    .as_str()
                    .unwrap()
                    .to_string(),
            ),
        )
        .item("json", AttributeValue::S(payload.to_string()));

    request.send().await.expect("PutItem failed");

    let resp = Response::builder()
        .status(201)
        .header("content-type", "application/json")
        .body(payload.to_string().into())
        .map_err(Box::new)?;
    Ok(resp)
}

fn get_key(meal: &Value) -> String {
    let uuid = json!(Uuid::new_v4().to_string());
    let id = meal.get("id").unwrap_or(&uuid);
    make_key(id.as_str().unwrap())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app_state = meal_init().await;
    run(service_fn(|event: Request| store_meal(&app_state, event)))
        .await
        .expect("faili");
    Ok(())
}
