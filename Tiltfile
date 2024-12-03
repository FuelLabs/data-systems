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

# Get deployment mode from environment variable, default to 'full'
# and configure values based on mode
config_mode = os.getenv('CLUSTER_MODE', 'full')

# Resource configurations
RESOURCES = {
    'publisher': {
        'name': 'local-fuel-streams-publisher',
        'ports': ['4000:4000', '8080:8080'],
        'labels': 'publisher',
        'config_mode': ['minimal', 'full']
    },
    'nats': {
        'name': 'local-nats',
        'ports': ['4222:4222'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    'nats-box': {
        'name': 'local-nats-box',
        'ports': [],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    'prometheus': {
        'name': 'local-fuel-local-prometheus',
        'ports': ['9090:9090'],
        'labels': 'monitoring',
        'config_mode': ['full']
    },
    'grafana': {
        'name': 'local-fuel-local-grafana',
        'ports': ['3000:3000'],
        'labels': 'monitoring',
        'config_mode': ['full']
    },
    'elasticsearch': {
        'name': 'local-fuel-local-elasticsearch',
        'ports': ['9200:9200', '9300:9300'],
        'labels': 'logging',
        'config_mode': ['full']
    },
    'kibana': {
        'name': 'local-fuel-local-kibana',
        'ports': ['5601:5601'],
        'labels': 'logging',
        'deps': ['elasticsearch'],
        'config_mode': ['full']
    }
}

# Deploy the Helm chart with values
helm_set_values = [
    name + ".enabled=" + str(config_mode in resource['config_mode']).lower()
    for name, resource in RESOURCES.items()
    if name not in ['publisher', 'nats', 'nats-box']  # Skip non-optional services
]

k8s_yaml(helm(
    'cluster/charts/fuel-local',
    name='local',
    namespace='fuel-local',
    values=[
        'cluster/charts/fuel-local/values.yaml',
        'cluster/charts/fuel-local/values-publisher-env.yaml',
    ],
    set=helm_set_values,
))

# Configure k8s resources
for name, resource in RESOURCES.items():
    if config_mode in resource['config_mode']:
        k8s_resource(
            resource['name'],
            new_name=name,
            port_forwards=resource['ports'],
            resource_deps=resource.get('deps', []),
            labels=resource['labels']
        )
