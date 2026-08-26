use std::path::PathBuf;

fn value(name: &str) -> Result<String, String> {
    let mut arguments = std::env::args();
    while let Some(argument) = arguments.next() {
        if argument == name {
            return arguments
                .next()
                .ok_or_else(|| format!("{name} requires a value"));
        }
    }
    Err(format!("missing required argument {name}"))
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph_path = PathBuf::from(value("--graph-path")?);
    let graph_id = value("--graph-id")?;
    let endpoint_file = PathBuf::from(value("--endpoint-file")?);
    let rules_path = PathBuf::from(value("--rules-path")?);
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async move {
            sitegraph_daemon::Server::bind(&graph_path, &graph_id, endpoint_file, &rules_path)
                .await?
                .run()
                .await
        })
        .await?;
    Ok(())
}
