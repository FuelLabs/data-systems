#!/usr/bin/env python
load('ext://restart_process', 'docker_build_with_restart')
load('ext://color', 'color')
load('ext://dotenv', 'dotenv')

analytics_settings(True) # Enable telemetry dialogue in web UI
disable_snapshots()      # Disable TiltCloud Snapshots
version_settings(True)   # Enable 'new version' banner

# Load environment variables from .env file
dotenv()

allow_k8s_contexts('minikube')

# Build sv-publisher
custom_build(
    ref='sv-publisher:latest',
    command=[
        './cluster/scripts/build_docker.sh',
        '--dockerfile', './cluster/docker/sv-publisher.Dockerfile'
    ],
    deps=[
        './src',
        './Cargo.toml',
        './Cargo.lock',
        './cluster/docker/sv-publisher.Dockerfile'
    ],
    live_update=[
        sync('./src', '/usr/src'),
        sync('./Cargo.toml', '/usr/src/Cargo.toml'),
        sync('./Cargo.lock', '/usr/src/Cargo.lock'),
        run('cargo build', trigger=['./src', './Cargo.toml', './Cargo.lock'])
    ],
    ignore=['./target']
)

# Build sv-consumer
custom_build(
    ref='sv-consumer:latest',
    image_deps=['sv-publisher:latest'],
    command=[
        './cluster/scripts/build_docker.sh',
        '--dockerfile', './cluster/docker/sv-consumer.Dockerfile'
    ],
    deps=[
        './src',
        './Cargo.toml',
        './Cargo.lock',
        './cluster/docker/sv-consumer.Dockerfile'
    ],
    live_update=[
        sync('./src', '/usr/src'),
        sync('./Cargo.toml', '/usr/src/Cargo.toml'),
        sync('./Cargo.lock', '/usr/src/Cargo.lock'),
        run('cargo build', trigger=['./src', './Cargo.toml', './Cargo.lock'])
    ],
    ignore=['./target']
)

# Build streamer ws image with proper configuration for Minikube
custom_build(
    ref='sv-webserver:latest',
    image_deps=['sv-consumer:latest', 'sv-publisher:latest'],
    command=[
        './cluster/scripts/build_docker.sh',
        '--dockerfile', './cluster/docker/sv-webserver.Dockerfile'
    ],
    deps=[
        './src',
        './Cargo.toml',
        './Cargo.lock',
        './cluster/docker/sv-webserver.Dockerfile'
    ],
    live_update=[
        sync('./src', '/usr/src'),
        sync('./Cargo.toml', '/usr/src/Cargo.toml'),
        sync('./Cargo.lock', '/usr/src/Cargo.lock'),
        run('cargo build', trigger=['./src', './Cargo.toml', './Cargo.lock'])
    ],
    ignore=['./target']
)

# Deploy the Helm chart with values from .env
# Get deployment mode from environment variable, default to 'full'
config_mode = os.getenv('CLUSTER_MODE', 'full')

# Resource configurations
RESOURCES = {
    'publisher': {
        'name': 'fuel-streams-publisher',
        'ports': ['8080:8080'],
        'labels': 'publisher',
        'config_mode': ['minimal', 'full'],
        'deps': ['fuel-streams-nats-core', 'fuel-streams-nats-publisher']
    },
    'consumer': {
        'name': 'fuel-streams-sv-consumer',
        'ports': ['8081:8080'],
        'labels': 'consumer',
        'config_mode': ['minimal', 'full'],
        'deps': ['fuel-streams-nats-core', 'fuel-streams-nats-publisher', 'fuel-streams-publisher']
    },
    'sv-webserver': {
        'name': 'sv-webserver',
        'ports': ['9003:9003'],
        'labels': 'ws',
        'config_mode': ['minimal', 'full'],
        'deps': ['fuel-streams-nats-core', 'fuel-streams-nats-publisher']
    },
    'nats-core': {
        'name': 'fuel-streams-nats-core',
        'ports': ['4222:4222', '6222:6222', '7422:7422'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    'nats-publisher': {
        'name': 'fuel-streams-nats-publisher',
        'ports': ['4333:4222', '6222:6222', '7433:7422'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full'],
        'deps': ['fuel-streams-nats-core']
    },
}

k8s_yaml(helm(
    'cluster/charts/fuel-streams',
    name='fuel-streams',
    namespace='fuel-streams',
    values=[
        'cluster/charts/fuel-streams/values.yaml',
        'cluster/charts/fuel-streams/values-local.yaml',
        'cluster/charts/fuel-streams/values-secrets.yaml'
    ]
))

# Configure k8s resources
for name, resource in RESOURCES.items():
    if config_mode in resource['config_mode']:
        k8s_resource(
            resource['name'],
            new_name=name,
            port_forwards=resource.get('ports', []),
            resource_deps=resource.get('deps', []),
            labels=resource['labels']
        )
