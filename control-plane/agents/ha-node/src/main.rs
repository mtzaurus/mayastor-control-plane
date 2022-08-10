use grpc::operations::ha_node::{client::ClusterAgentClient, traits::ClusterAgentOperations};
use http::Uri;
use once_cell::sync::OnceCell;
use opentelemetry::KeyValue;
use structopt::StructOpt;
use utils::{
    package_description, version_info_str, DEFAULT_CLUSTER_AGENT_SERVER_ADDR,
    DEFAULT_NODE_AGENT_SERVER_ADDR, NVME_PATH_CHECK_PERIOD,
};

#[macro_use]
extern crate tracing;

mod detector;
mod path_provider;

use detector::PathFailureDetector;

/// TODO
#[derive(Debug, StructOpt)]
#[structopt(name = package_description!(), version = version_info_str!())]
pub struct Cli {
    /// HA Cluster Agent URL or address to connect to the services.
    #[structopt(long, short, default_value = DEFAULT_CLUSTER_AGENT_SERVER_ADDR)]
    cluster_agent: Uri,

    /// Node name(spec.nodeName). This must be the same as provided in csi-node
    #[structopt(short, long)]
    node_name: String,

    /// IP address and port for the ha node-agent to listen on
    #[structopt(short, long, default_value = DEFAULT_NODE_AGENT_SERVER_ADDR)]
    grpc_endpoint: Uri,

    /// Add process service tags to the traces
    #[structopt(short, long, env = "TRACING_TAGS", value_delimiter=",", parse(try_from_str = utils::tracing_telemetry::parse_key_value))]
    tracing_tags: Vec<KeyValue>,

    /// Path failure detection period.
    #[structopt(short, long, env = "DETECTION_PERIOD", default_value = NVME_PATH_CHECK_PERIOD)]
    detection_period: humantime::Duration,
}

static CLUSTER_AGENT_CLIENT: OnceCell<ClusterAgentClient> = OnceCell::new();

fn cluster_agent_client() -> &'static ClusterAgentClient {
    CLUSTER_AGENT_CLIENT
        .get()
        .expect("HA Cluster-Agent client should have been initialized")
}

impl Cli {
    fn args() -> Self {
        Cli::from_args()
    }
}

#[tokio::main]
async fn main() {
    let cli_args = Cli::args();

    utils::print_package_info!();

    utils::tracing_telemetry::init_tracing(
        "agent-ha-node",
        cli_args.tracing_tags.clone(),
        None,
        //cli_args.jaeger.clone(),
    );

    // Instantinate path failure detector.
    let detector = PathFailureDetector::new(&cli_args)
        .await
        .expect("Failed to initialize path failure detector");

    CLUSTER_AGENT_CLIENT
        .set(ClusterAgentClient::new(cli_args.cluster_agent, None).await)
        .ok()
        .expect("Expect to be initialized only once");

    cluster_agent_client()
        .register(cli_args.node_name.clone(), cli_args.grpc_endpoint)
        .await
        .unwrap();

    detector
        .start()
        .await
        .expect("Failed to start NVMe path failure detector");
}
