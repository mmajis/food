use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let func = service_fn(func);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn func(_event: LambdaEvent<Value>) -> Result<Value, Error> {
    let (event, _context) = _event.into_parts();
    //let first_name = event["firstName"].as_str().unwrap_or("world");
    println!("{}", event);
    Ok(json!({ "message": format!("Hello, {}!", "World") }))
}
