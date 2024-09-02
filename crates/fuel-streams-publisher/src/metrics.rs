use prometheus::{
    HistogramOpts,
    HistogramVec,
    IntCounter,
    IntCounterVec,
    IntGauge,
    Opts,
    Registry,
};

#[derive(Clone, Debug, Default)]
pub struct PublisherMetrics {
    pub registry: Registry,
}

impl PublisherMetrics {
    pub fn new() -> anyhow::Result<Self> {
        let incoming_requests: IntCounter =
            IntCounter::new("incoming_requests", "Incoming Requests")
                .expect("metric can be created");
        let connected_clients: IntGauge =
            IntGauge::new("connected_clients", "Connected Clients")
                .expect("metric can be created");
        let response_code_collector: IntCounterVec = IntCounterVec::new(
            Opts::new("response_code", "Response Codes"),
            &["env", "statuscode", "type"],
        )
        .expect("metric can be created");
        let response_time_collector: HistogramVec = HistogramVec::new(
            HistogramOpts::new("response_time", "Response Times"),
            &["env"],
        )
        .expect("metric can be created");

        let registry: Registry = Registry::new();

        registry.register(Box::new(incoming_requests.clone()))?;

        registry.register(Box::new(connected_clients.clone()))?;

        registry.register(Box::new(response_code_collector.clone()))?;

        registry.register(Box::new(response_time_collector.clone()))?;

        Ok(Self { registry })
    }
}
