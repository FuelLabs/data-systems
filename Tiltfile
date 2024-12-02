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

# project namespace
namespace = "fuel-local"

# load fuel helm chart
fuel_chart_name = "fuel"
fuel_chart_dir = "cluster/charts/fuel-local"
fuel_values = "cluster/charts/fuel-local/values.yaml"
k8s_yaml(helm(fuel_chart_dir, name=fuel_chart_name, namespace=namespace, values=[fuel_values], set=[]))

# load nats helm chart
nats_chart_name = "nats"
nats_chart_dir = "cluster/charts/fuel-nats"
nats_values = "cluster/charts/fuel-nats/values.yaml"
k8s_yaml(helm(nats_chart_dir, name=nats_chart_name, namespace=namespace, values=[nats_values], set=[]))

# load surrealdb helm chart
surrealdb_chart_name = "surrealdb"
surrealdb_chart_dir = "cluster/charts/fuel-surrealdb"
surrealdb_values = "cluster/charts/fuel-surrealdb/values.yaml"
k8s_yaml(helm(surrealdb_chart_dir, name=surrealdb_chart_name, namespace=namespace, values=[surrealdb_values], set=[]))

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
    "fuel-publisher": ["4000:4000", "8080:8080"],
    "nats": ["4222:4222"],
    "nats-box": [],
}
deps = {
    "monitoring": [],
    "surrealdb": [],
    "elasticsearch": [],
    "kibana": ["elasticsearch"],
    "fuel-publisher": [],
    "nats": [],
    "nats-box": [],
}

k8s_resource("monitoring", port_forwards=ports["monitoring"], resource_deps=deps["monitoring"], labels="monitoring")
k8s_resource("surrealdb", port_forwards=ports["surrealdb"], resource_deps=deps["surrealdb"], labels="indexer")
k8s_resource("elasticsearch", port_forwards=ports["elasticsearch"], resource_deps=deps["elasticsearch"], labels="logging")
k8s_resource("kibana", port_forwards=ports["kibana"], resource_deps=deps["kibana"], labels="logging")
k8s_resource("fuel-publisher", port_forwards=ports["fuel-publisher"], resource_deps=deps["fuel-publisher"], labels="publisher")
k8s_resource("nats", port_forwards=ports["nats"], resource_deps=deps["nats"], labels="nats")
k8s_resource("nats-box", port_forwards=ports["nats-box"], resource_deps=deps["nats-box"], labels="nats")
