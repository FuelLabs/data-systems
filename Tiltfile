#!/usr/bin/env python
load('ext://restart_process', 'docker_build_with_restart')
load('ext://color', 'color')

analytics_settings(True) # Enable telemetry dialogue in web UI
disable_snapshots()      # Disable TiltCloud Snapshots
version_settings(True)   # Enable 'new version' banner

# parse Tiltfile config
config.define_string(name="env", args=False, usage="This argument defines the build env (dev or release)")
settings = config.parse()
environment=settings.get('env', "dev") #dev or release

# load ingresses
k8s_yaml('./cluster/charts/fuel-local/crds/traefik-resources.yaml')
k8s_kind('IngressRoute')

# disable unused image
update_settings(suppress_unused_image_warnings=["k8s-tools:latest"])

# load helm charts
namespace = "fuel-local"
chart_name = "fuel"
chart_dir = "cluster/charts/fuel-local"
values = "cluster/values/fuel-local.yaml"
overrides = [
    "monitoring.enabled=true",
    "grafana.image=grafana/grafana",
    "grafana.tag=7.2.1",
    "prometheus.image=prom/prometheus",
    "prometheus.tag=v2.22.0",

    "elasticsearch.enabled=true",
    "elasticsearch.image=docker.elastic.co/elasticsearch/elasticsearch",
    "elasticsearch.tag=7.10.2",

    "kibana.enabled=true",
    "kibana.image=docker.elastic.co/kibana/kibana",
    "kibana.tag=7.6.2",

    "surrealdb.enabled=true",

    "fuelPublisher.enabled=true",
]

k8s_yaml(helm(chart_dir, name=chart_name, namespace=namespace, values=[values], set=overrides))

# build publisher image
# ref = 'fuel-publisher:{}'
# command = 'make build-fuel-publisher-{} && docker tag fuel-publisher:{} $EXPECTED_REF'.format(environment, environment)
# deps = ["./fuel-publisher/Cargo.lock", "./fuel-publisher/Cargo.toml", "./fuel-publisher/src", "./fuel-publisher/target/release/**/*"]
# custom_build(ref=ref, command=command, deps=deps)

# build k8s tools (tag is always latest!)
ref = "k8s-tools:latest"
command = "make build-k8s-tools && docker tag k8s-tools:latest $EXPECTED_REF"
custom_build(ref=ref, command=command, deps=[])

# k8s resources
ports = {
    "monitoring": ["9090:9090", "3000:3000"],
    "surrealdb": ["8000:8000", "8001:8001"],
    "elasticsearch": ["9200:9200", "9300:9300"],
    "kibana": ["5601:5601"],
    "fuel-publisher": [],
}
deps = {
    "monitoring": [],
    "surrealdb": [],
    "elasticsearch": [],
    "kibana": ["elasticsearch"],
    "fuel-publisher": [],
}

k8s_resource("monitoring", port_forwards=ports["monitoring"], resource_deps=deps["monitoring"], labels="monitoring")
k8s_resource("surrealdb", port_forwards=ports["surrealdb"], resource_deps=deps["surrealdb"], labels="indexer")
k8s_resource("elasticsearch", port_forwards=ports["elasticsearch"], resource_deps=deps["elasticsearch"], labels="logging")
k8s_resource("kibana", port_forwards=ports["kibana"], resource_deps=deps["kibana"], labels="logging")
k8s_resource("fuel-publisher", port_forwards=ports["fuel-publisher"], resource_deps=deps["fuel-publisher"], labels="publisher")
