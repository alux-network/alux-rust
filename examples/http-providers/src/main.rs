use http_providers::{run_until, say_nothing};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), http_providers::BoxError> {
    // Every provider says what it is doing through this example, not through a framework's logger.
    say_nothing();

    run_until(async {
        tokio::signal::ctrl_c().await.expect("installs Ctrl-C handler");
    })
    .await
}
