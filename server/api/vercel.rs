mod lambda;

use server::setup_app;
use vercel_runtime::{process_request, process_response, run_service, Error, ServiceBuilder};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let app = setup_app("./dist".to_owned()).await;

    let handler = ServiceBuilder::new()
        .map_request(process_request)
        .map_response(process_response)
        .layer(lambda::LambdaLayer::default())
        .service(app);

    run_service(handler).await
}
