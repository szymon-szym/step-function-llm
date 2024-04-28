
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};

mod lib;

use lib::*;


async fn function_handler(event: LambdaEvent<EventInput>) -> Result<Vec<ResultMessage>, Error> {

    let req = event.payload;
    
    let result_messages = handle_input(req);
    

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(result_messages)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(function_handler)).await
}
