use common::{
    aws_sdk_dynamodb::types::AttributeValue,
    common::{meal_init, AppState, ErrorResponse},
    serde_dynamo::aws_sdk_dynamodb_0_26::from_items,
};
use lambda_http::{
    aws_lambda_events::serde_json::{from_str, to_string, Value},
    run, service_fn, Body, Error, Request, RequestExt, Response,
};
use tracing::info;

async fn search_meal(state: &AppState, event: Request) -> Result<Response<Body>, Error> {
    info!("{:#?}", event);

    let params = event.query_string_parameters();
    info!("{:#?}", params);
    let q = params.first("q");
    if let Some(query) = q {
        let request = state
            .dynamo_client
            .scan()
            .table_name(&state.table_name)
            .set_filter_expression(Some("contains(#name, :value)".to_string()))
            .expression_attribute_names("#name", "name")
            .expression_attribute_values(":value", AttributeValue::S(query.to_string()));

        let response = request.send().await.expect("DynamoDB operation failed");

        info!("{:#?}", response);

        let items = response.items().unwrap();
        let values: Vec<Value> = from_items(items.to_owned()).unwrap();
        let values_mapped: Vec<Value> = values
            .into_iter()
            .map(|x| from_str(x["json"].as_str().unwrap()).unwrap())
            .collect();

        info!("{:#?}", values_mapped);

        let value_arr = Value::Array(values_mapped);

        let resp = Response::builder()
            .status(200)
            .header("content-type", "application/json")
            .body(value_arr.to_string().into())
            .map_err(Box::new)?;
        Ok(resp)
    } else {
        let resp = Response::builder()
            .status(400)
            .header("content-type", "application/json")
            .body(
                to_string(&ErrorResponse {
                    message: String::from("Query parameter 'q' is required"),
                })
                .unwrap()
                .into(),
            )
            .map_err(Box::new)?;
        Ok(resp)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app_state = meal_init().await;
    run(service_fn(|event: Request| search_meal(&app_state, event)))
        .await
        .expect("faili");
    Ok(())
}
