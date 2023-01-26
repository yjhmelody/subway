use std::net::SocketAddr;

use jsonrpsee::server::{RpcModule, ServerBuilder};
use tokio::task::JoinHandle;

use crate::{client::Client, config::Config};

// TODO: https://github.com/paritytech/jsonrpsee/issues/985
fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub async fn start_server(
    config: &Config,
    client: Client,
) -> anyhow::Result<(SocketAddr, JoinHandle<()>)> {
    let service_builder = tower::ServiceBuilder::new();

    let server = ServerBuilder::default()
        .set_middleware(service_builder)
        .build((config.listen_address.as_str(), config.port))
        .await?;

    let mut module = RpcModule::new(client);

    for method in &config.rpcs.methods {
        let method_name = string_to_static_str(method.method.clone());
        module.register_async_method(method_name, move |params, ctx| async move {
            ctx.request(method_name, params).await
        })?;
    }

    let addr = server.local_addr()?;
    let handle = server.start(module)?;

    let handle = tokio::spawn(handle.stopped());

    Ok((addr, handle))
}
