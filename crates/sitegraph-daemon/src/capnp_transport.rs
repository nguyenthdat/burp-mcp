use super::*;
use capnp_rpc::{RpcSystem, rpc_twoparty_capnp, twoparty};
use futures::AsyncReadExt as _;
use tokio_util::compat::TokioAsyncReadCompatExt as _;

pub(crate) struct Service {
    pub(crate) graph: Arc<SiteGraph>,
    pub(crate) token: Arc<str>,
}

impl sitegraph_capnp::sitegraph::Server for Service {
    async fn call(
        self: capnp::capability::Rc<Self>,
        params: sitegraph_capnp::sitegraph::CallParams,
        mut results: sitegraph_capnp::sitegraph::CallResults,
    ) -> std::result::Result<(), capnp::Error> {
        let params = params.get()?;
        let token = params.get_token()?.to_string()?;
        let mut output = results.get();
        if token != self.token.as_ref() {
            output.set_ok(false);
            output.set_error("unauthorized");
            return Ok(());
        }
        let task = tasks::Task::read_capnp(params.get_task()?)
            .map_err(|error| capnp::Error::failed(error.to_string()))?;
        let result = match tasks::dispatch(&self.graph, task).await {
            Ok(response) => response
                .encode()
                .map_err(|error| capnp::Error::failed(error.to_string()))?,
            Err(error) => {
                output.set_ok(false);
                output.set_error(error.to_string());
                return Ok(());
            }
        };
        if result.len() > MAX_FRAME_BYTES {
            output.set_ok(false);
            output.set_error("response exceeds 128 MiB");
        } else {
            output.set_ok(true);
            output.set_payload(&result);
        }
        Ok(())
    }
}

pub(crate) async fn serve_connection(
    stream: TcpStream,
    graph: Arc<SiteGraph>,
    token: Arc<str>,
) -> Result<()> {
    stream.set_nodelay(true)?;
    let (reader, writer) = stream.compat().split();
    let network = Box::new(twoparty::VatNetwork::new(
        futures::io::BufReader::new(reader),
        futures::io::BufWriter::new(writer),
        rpc_twoparty_capnp::Side::Server,
        Default::default(),
    ));
    let bootstrap: sitegraph_capnp::sitegraph::Client =
        capnp_rpc::new_client(Service { graph, token });
    RpcSystem::new(network, Some(bootstrap.client))
        .await
        .map_err(Error::Capnp)
}
