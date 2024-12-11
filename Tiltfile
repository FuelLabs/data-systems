#!/usr/bin/env python
load('ext://restart_process', 'docker_build_with_restart')
load('ext://color', 'color')
load('ext://dotenv', 'dotenv')

analytics_settings(True) # Enable telemetry dialogue in web UI
disable_snapshots()      # Disable TiltCloud Snapshots
version_settings(True)   # Enable 'new version' banner

# Load environment variables from .env file
dotenv()

# Build images with proper configuration for Minikube
IMAGES = {
    # 'fuel-streams-publisher': 'fuel-streams-publisher',
    'sv-emitter': 'sv-emitter',
    'sv-consumer': 'sv-consumer'
}

for ref, image_name in IMAGES.items():
    custom_build(
        ref=f'{ref}:latest',
        command=[f'IMAGE_NAME={image_name}', './cluster/scripts/build_docker.sh'],
        deps=[
            './src',
            './Cargo.toml',
            './Cargo.lock',
            f'./cluster/docker/{image_name}.Dockerfile'
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
    # 'publisher': {
    #     'name': 'fuel-streams-publisher',
    #     'ports': ['4000:4000', '8080:8080'],
    #     'labels': 'publisher',
    #     'config_mode': ['minimal', 'full']
    # },
    'emitter': {
        'name': 'sv-emitter',
        'ports': ['8081:8081'],
        'labels': 'emitter',
        'config_mode': ['minimal', 'full']
    },
    'consumer': {
        'name': 'sv-consumer',
        'ports': ['8082:8082'],
        'labels': 'consumer',
        'config_mode': ['minimal', 'full']
    },
    'nats-core': {
        'name': 'fuel-streams-nats-core',
        'ports': ['4222:4222', '8222:8222'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    'nats-client': {
        'name': 'fuel-streams-nats-client',
        'ports': ['4223:4222', '8443:8443'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    'nats-publisher': {
        'name': 'fuel-streams-nats-publisher',
        'ports': ['4224:4222'],
        'labels': 'nats',
        'config_mode': ['minimal', 'full']
    },
    # 'grafana': {
    #     'name': 'fuel-streams-grafana',
    #     'ports': ['3000:3000'],
    #     'labels': 'monitoring',
    #     'config_mode': ['minimal', 'full']
    # },
    # 'prometheus-operator': {
    #     'name': 'fuel-streams-prometheus-operator',
    #     'labels': 'monitoring',
    #     'config_mode': ['minimal', 'full']
    # },
    # 'kube-state-metrics': {
    #     'name': 'fuel-streams-kube-state-metrics',
    #     'labels': 'monitoring',
    #     'config_mode': ['minimal', 'full']
    # },
    # 'node-exporter': {
    #     'name': 'fuel-streams-prometheus-node-exporter',
    #     'labels': 'monitoring',
    #     'config_mode': ['minimal', 'full']
    # }
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
