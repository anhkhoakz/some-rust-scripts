use pingora_load_balancing::{LoadBalancer, health_check};

use pingora_core::Result;
use pingora_core::server::Server;
use pingora_core::server::configuration::Opt;
use pingora_core::upstreams::peer::HttpPeer;
use pingora_load_balancing::{LoadBalancer, health_check, selection::RoundRobin};
use pingora_proxy::{ProxyHttp, Session};

fn main() {
    let opt = Opt:parse();
    let mut my_server = Server::new().unwrap();
    let mut upstreams = LoadBalancer::try_from_iter(["1.1.1.1:443", "1.0.0.1:433"]).unwrap();
    let hc = health_check::TcpHealthCheck::new();
    upstreams.set_health_check(hc);
}
