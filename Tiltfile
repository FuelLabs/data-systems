#!/usr/bin/env python
load('ext://restart_process', 'docker_build_with_restart')
load('ext://color', 'color')

analytics_settings(True) # Disable telemetry dialogue in web UI
disable_snapshots()      # Disable TiltCloud Snapshots
version_settings(True)   # Disable 'new version' banner

# parse Tiltfile config
config.define_string(name="env", args=False, usage="This argument defines the build env (dev or release)")
settings = config.parse()
environment=settings.get('env', "dev") #dev or release

# load ingresses
k8s_yaml('./cluster/charts/fuel-local/crds/traefik-resources.yaml')
k8s_kind('IngressRoute')

# load helm charts
namespace = "fuel-local"
chart_name = "fuel"
chart_dir = "cluster/charts/fuel-local"
values = "cluster/values/fuel-local.yaml"
overrides = [
    "nearApi.enabled=true",
    "nearApi.image=near-api",
    'nearApi.tag={}'.format(environment),
    'nearApi.env={}'.format(environment),

    "gqlApi.enabled=true",
    "gqlApi.image=gql-api",
    'gqlApi.tag={}'.format(environment),
    'gqlApi.env={}'.format(environment),

    "monitoring.enabled=true",
    "grafana.image=grafana/grafana",
    "grafana.tag=7.2.1",
    "prometheus.image=prom/prometheus",
    "prometheus.tag=v2.22.0",

    "elasticsearch.enabled=true",
    "elasticsearch.image=docker.elastic.co/elasticsearch/elasticsearch",
    "elasticsearch.tag=7.10.2",

    "jaeger.enabled=true",
    "jaeger.image=jaegertracing/all-in-one",
    "jaeger.tag=latest",

    "kibana.enabled=true",
    "kibana.image=docker.elastic.co/kibana/kibana",
    "kibana.tag=7.6.2",

    "surrealdb.enabled=true",
]

k8s_yaml(helm(chart_dir, name=chart_name, namespace=namespace, values=[values], set=overrides))

# build gqlApi image
ref = 'gql-api:{}'.format(environment)
command = 'make build-gql-api-{} && docker tag gql-api:{} $EXPECTED_REF'.format(environment, environment)
deps = ["./gql-api/Cargo.lock", "./gql-api/Cargo.toml", "./gql-api/src", "./gql-api/target/release/**/*"]
custom_build(ref=ref, command=command, deps=deps)

# build nearApi image
ref = 'near-api:{}'.format(environment)
command = 'make build-near-api-{} && docker tag near-api:{} $EXPECTED_REF'.format(environment, environment)
deps = ["./near-api/package-lock.json", "./near-api/package.json", "./**/*"]
custom_build(ref=ref, command=command, deps=deps)

# build k8s tools (tag is always latest!)
ref = "k8s-tools:latest"
command = "make build-k8s-tools && docker tag k8s-tools:latest $EXPECTED_REF"
custom_build(ref=ref, command=command, deps=deps)

# k8s resources
ports = {
    "monitoring": ["9090:9090", "3000:3000"],
    "surrealdb": ["8000:8000", "8001:8001"],
    "jaeger": ["5775:5775", "6831:6831", "6832:6832", "5778:5778", "16686:16686", "14268:14268", "9411:9411"],
    "elasticsearch": ["9200:9200", "9300:9300"],
    "kibana": ["5601:5601"],
    "gql-api": ["3200:8080"],
    "near-api": ["7000:50051"],
}
deps = {
    "monitoring": [],
    "surrealdb": [],
    "elasticsearch": [],
    "jaeger": [],
    "kibana": ["elasticsearch"],
    "dashboard": [],
    "gql-api": ["monitoring", "elasticsearch" ],
    "near-api": [],
}

k8s_resource("gql-api", port_forwards=ports["gql-api"], resource_deps=deps["gql-api"], labels="data-streams")
k8s_resource("near-api", port_forwards=ports["near-api"], resource_deps=deps["near-api"], labels="data-streams")
k8s_resource("monitoring", port_forwards=ports["monitoring"], resource_deps=deps["monitoring"], labels="monitoring")
k8s_resource("surrealdb", port_forwards=ports["surrealdb"], resource_deps=deps["surrealdb"], labels="indexer")
k8s_resource("elasticsearch", port_forwards=ports["elasticsearch"], resource_deps=deps["elasticsearch"], labels="logging")
k8s_resource("jaeger", port_forwards=ports["jaeger"], resource_deps=deps["jaeger"], labels="monitoring")
k8s_resource("kibana", port_forwards=ports["kibana"], resource_deps=deps["kibana"], labels="logging")
