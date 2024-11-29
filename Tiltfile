#!/usr/bin/env python
load('ext://restart_process', 'docker_build_with_restart')
load('ext://color', 'color')
load('ext://dotenv', 'dotenv')

analytics_settings(True) # Enable telemetry dialogue in web UI
disable_snapshots()      # Disable TiltCloud Snapshots
version_settings(True)   # Enable 'new version' banner

# Load environment variables from .env file
dotenv()

# Build publisher image with proper configuration for Minikube
custom_build(
    ref='fuel-streams-publisher:latest',
    command=['./cluster/build_publisher.sh'],
    deps=[
        './src',
        './Cargo.toml',
        './Cargo.lock',
        './docker/fuel-streams-publisher.Dockerfile'
    ],
    live_update=[
        sync('./src', '/usr/src'),
        sync('./Cargo.toml', '/usr/src/Cargo.toml'),
        sync('./Cargo.lock', '/usr/src/Cargo.lock'),
        run('cargo build', trigger=['./src', './Cargo.toml', './Cargo.lock'])
    ],
    skips_local_docker=True,
    ignore=['./target']
)

# Deploy the Helm chart with values from .env
k8s_yaml(helm(
    'cluster/charts/fuel-local',
    name='local',
    namespace='fuel-local',
    values=[
        'cluster/charts/fuel-local/values.yaml',
        'cluster/charts/fuel-local/values-publisher.yaml',
        'cluster/charts/fuel-local/values-publisher-env.yaml',
    ]
))

# load nats helm chart
nats_chart_dir = "cluster/charts/fuel-nats"
nats_values = "cluster/charts/fuel-nats/values.yaml"
k8s_yaml(helm(nats_chart_dir, name="nats", namespace='fuel-local', values=[nats_values]))

# load surrealdb helm chart
surrealdb_chart_dir = "cluster/charts/fuel-surrealdb"
surrealdb_values = "cluster/charts/fuel-surrealdb/values.yaml"
k8s_yaml(helm(surrealdb_chart_dir, name="surrealdb", namespace='fuel-local', values=[surrealdb_values]))

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
    "fuel-streams-publisher": ["4000:4000", "9000:9000", "30333:30333"],
    "nats": ["4222:4222"],
    "nats-box": [],
}
deps = {
    "monitoring": [],
    "surrealdb": [],
    "elasticsearch": [],
    "kibana": ["elasticsearch"],
    "fuel-streams-publisher": [],
    "nats": [],
    "nats-box": [],
}

k8s_resource("monitoring", port_forwards=ports["monitoring"], resource_deps=deps["monitoring"], labels="monitoring")
k8s_resource("surrealdb", port_forwards=ports["surrealdb"], resource_deps=deps["surrealdb"], labels="indexer")
k8s_resource("elasticsearch", port_forwards=ports["elasticsearch"], resource_deps=deps["elasticsearch"], labels="logging")
k8s_resource("kibana", port_forwards=ports["kibana"], resource_deps=deps["kibana"], labels="logging")
k8s_resource("local-fuel-streams-publisher",
    new_name="streams-publisher",  # Override the display name
    resource_deps=deps["fuel-streams-publisher"],
    port_forwards=ports["fuel-streams-publisher"],
    labels="publisher"
)
k8s_resource("nats", port_forwards=ports["nats"], resource_deps=deps["nats"], labels="nats")
k8s_resource("nats-box", port_forwards=ports["nats-box"], resource_deps=deps["nats-box"], labels="nats")
