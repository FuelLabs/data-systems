#!/usr/bin/env python
load('ext://restart_process', 'docker_build_with_restart')
load('ext://color', 'color')
load('ext://dotenv', 'dotenv')

analytics_settings(True)  # Enable telemetry dialogue in web UI
disable_snapshots()  # Disable TiltCloud Snapshots
version_settings(True)  # Enable 'new version' banner

# Load environment variables from .env file
dotenv()

allow_k8s_contexts('minikube')

# Build sv-publisher
custom_build(
    ref='sv-dune:latest',
    command=[
        './cluster/scripts/build_docker.sh',
        '--dockerfile', './cluster/docker/sv-dune.Dockerfile'
    ],
    deps=[
        './src',
        './Cargo.toml',
        './Cargo.lock',
        './cluster/docker/sv-dune.Dockerfile'
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
    'dune': {
        'name': 'fuel-streams-sv-dune',
        'ports': ['8080:8080'],
        'labels': 'dune',
        'config_mode': ['minimal', 'full'],
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
