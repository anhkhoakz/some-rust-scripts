use async_trait::async_trait;
use pingora::prelude::*;
use std::sync::Arc;

pub struct LB(Arc<LoadBalancer<RoundRobin>>);

#[async_trait]
impl ProxyHttp for LB {
    type CTX = ();

    fn new_ctx(&self) -> () {
        ()
    }

    async fn upstream_peer(&self, _session: &mut Session, _ctx: &mut ()) -> Result<Box<HttpPeer>> {
        let upstream: pingora_load_balancing::Backend = self.0.select(b"", 256).unwrap();

        println!("upstream peer is:, {upstream:?}");
        let peer: Box<HttpPeer> =
            Box::new(HttpPeer::new(upstream, true, "one.one.one.one".to_string()));
        Ok(peer)
    }

    async fn upstream_request_filter(
        &self,
        _session: &mut Session,
        upstream_request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        upstream_request.insert_header("Host", "one.one.one.one")?;
        Ok(())
    }
}

fn main() {
    let mut my_server: Server = Server::new(Some(Opt::parse_args())).unwrap();
    my_server.bootstrap();

    let mut upstreams: LoadBalancer<
        pingora_load_balancing::selection::weighted::Weighted<
            pingora_load_balancing::selection::algorithms::RoundRobin,
        >,
    > = LoadBalancer::try_from_iter(["1.1.1.1:443", "1.0.0.1:443", "127.0.0.1:343"]).unwrap();
    let hc: Box<TcpHealthCheck> = TcpHealthCheck::new();
    upstreams.set_health_check(hc);
    upstreams.health_check_frequency = Some(std::time::Duration::from_secs(1));
    let background: pingora::services::background::GenBackgroundService<
        LoadBalancer<
            pingora_load_balancing::selection::weighted::Weighted<
                pingora_load_balancing::selection::algorithms::RoundRobin,
            >,
        >,
    > = background_service("health check", upstreams);
    let upstreams: Arc<
        LoadBalancer<
            pingora_load_balancing::selection::weighted::Weighted<
                pingora_load_balancing::selection::algorithms::RoundRobin,
            >,
        >,
    > = background.task();

    let mut lb: pingora::services::listening::Service<pingora_proxy::HttpProxy<LB>> =
        http_proxy_service(&my_server.configuration, LB(upstreams));
    lb.add_tcp("0.0.0.0:6188");

    my_server.add_service(background);

    my_server.add_service(lb);
    my_server.run_forever();
}
