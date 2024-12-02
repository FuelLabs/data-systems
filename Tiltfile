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
    command=['./cluster/scripts/build_publisher.sh'],
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
        'cluster/charts/fuel-local/values-publisher-env.yaml',
    ]
))

# k8s resources
ports = {
    "monitoring": ["9090:9090", "3000:3000"],
    "elasticsearch": ["9200:9200", "9300:9300"],
    "kibana": ["5601:5601"],
    "publisher": ["4000:4000", "8080:8080"],
    "nats": ["4222:4222"],
    "nats-box": [],
}

deps = {
    "monitoring": [],
    "elasticsearch": [],
    "kibana": ["elasticsearch"],
    "publisher": [],
    "nats": [],
    "nats-box": [],
}

k8s_resource("monitoring",
    port_forwards=ports["monitoring"],
    resource_deps=deps["monitoring"],
    labels="monitoring"
)
k8s_resource("elasticsearch",
    port_forwards=ports["elasticsearch"],
    resource_deps=deps["elasticsearch"],
    labels="logging"
)
k8s_resource("kibana",
    port_forwards=ports["kibana"],
    resource_deps=deps["kibana"],
    labels="logging"
)
k8s_resource("local-fuel-streams-publisher",
    new_name="publisher",  # Override the display name
    resource_deps=deps["publisher"],
    port_forwards=ports["publisher"],
    labels="publisher"
)
k8s_resource("local-nats",
    new_name="nats",
    port_forwards=ports["nats"],
    resource_deps=deps["nats"],
    labels="nats"
)
k8s_resource("local-nats-box",
    new_name="nats-box",
    port_forwards=ports["nats-box"],
    resource_deps=deps["nats-box"],
    labels="nats"
)
