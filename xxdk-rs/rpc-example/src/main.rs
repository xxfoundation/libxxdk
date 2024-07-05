#[tokio::main]
async fn main() {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Display the thread ID an event was recorded on
        .with_thread_ids(true)
        // Don't display the event's target (module path)
        .with_target(false)
        .with_max_level(tracing::Level::TRACE)
        // Build the subscriber
        .finish();
    // use that subscriber to process traces emitted after this point
    let _ = tracing::subscriber::set_global_default(subscriber);
    if let Err(err) = rpc_example::run().await {
        eprintln!("Error: {err}");
        std::process::exit(-1);
    }
}
