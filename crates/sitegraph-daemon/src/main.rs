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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let graph_path = PathBuf::from(value("--graph-path")?);
    let graph_id = value("--graph-id")?;
    let endpoint_file = PathBuf::from(value("--endpoint-file")?);
    sitegraph_daemon::Server::bind(&graph_path, &graph_id, endpoint_file)
        .await?
        .run()
        .await?;
    Ok(())
}
