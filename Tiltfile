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

# Build sv-emitter
custom_build(
    ref='sv-emitter:latest',
    command=[
        './cluster/scripts/build_docker.sh',
        'sv-emitter',
        '--build-arg', 'PACKAGE_NAME=sv-emitter',
        '--build-arg', 'PORT=4000',
        '--build-arg', 'P2P_PORT=30333'
    ],
    deps=[
        './src',
        './Cargo.toml',
        './Cargo.lock',
        './cluster/docker/publisher.Dockerfile'
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

# Build sv-consumer
custom_build(
    ref='sv-consumer:latest',
    command=[
        './cluster/scripts/build_docker.sh',
        'sv-consumer',
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
    skips_local_docker=True,
    ignore=['./target']
)

# Get deployment mode from environment variable, default to 'full'
config_mode = os.getenv('CLUSTER_MODE', 'full')

# Resource configurations
RESOURCES = {
    'publisher': {
        'name': 'fuel-streams-publisher',
        'ports': ['4000:4000', '8080:8080'],
        'labels': 'publisher',
        'config_mode': ['minimal', 'full']
    },
    'consumer': {
        'name': 'fuel-streams-sv-consumer',
        'ports': ['8082:8082'],
        'labels': 'consumer',
        'config_mode': ['minimal', 'full']
    },
    'nats-core': {
        'name': 'fuel-streams-nats-core',
        'ports': ['4222:4222', '6222:6222', '7422:7422'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    'nats-client': {
        'name': 'fuel-streams-nats-client',
        'ports': ['14222:4222', '17422:7422', '8443:8443'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    'nats-publisher': {
        'name': 'fuel-streams-nats-publisher',
        'ports': ['24222:4222', '27422:7422'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    }
}

k8s_yaml(helm(
    'cluster/charts/fuel-streams',
    name='fuel-streams',
    namespace='fuel-streams',
    values=[
        'cluster/charts/fuel-streams/values-publisher-secrets.yaml',
        'cluster/charts/fuel-streams/values.yaml'
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
