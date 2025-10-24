# Fuel Streams Helm Chart

This Helm chart deploys the complete Fuel Streams data processing pipeline for Kubernetes, enabling real-time blockchain data indexing, processing, and serving.

## Chart Structure

The Fuel Streams Helm chart follows a structured organization to manage the deployment of multiple services. Here's an overview of the chart's directory structure:

```
fuel-streams/
├── Chart.yaml              # Chart metadata and dependencies
├── Chart.lock              # Locked dependencies
├── values.yaml             # Default configuration values
├── values-local.yaml       # Configuration for local development
├── values-secrets.yaml     # Template for secret values
├── .helmignore             # Files to ignore when packaging
├── CHANGELOG.md            # Chart version history
├── charts/                 # Dependency charts
│   ├── kube-prometheus-stack-27.1.0.tgz  # Prometheus monitoring
│   └── nats-1.3.2.tgz                    # NATS message broker
├── dashboards/             # Grafana dashboards
│   ├── api-metrics.json       # API service dashboard
│   ├── consumer-metrics.json  # Consumer service dashboard
│   ├── publisher-metrics.json # Publisher service dashboard
│   └── webserver-metrics.json # Webserver service dashboard
├── templates/              # Kubernetes resource templates
│   ├── _helpers.tpl           # Helper functions
│   ├── _blocks-*.tpl          # Reusable template blocks
│   ├── _*.yaml                # Common resource templates
│   ├── common-config.yaml     # ConfigMap for shared configuration
│   ├── namespace.yaml         # Namespace definition
│   ├── service-account.yaml   # Service account definition
│   ├── api/                   # API service resources
│   │   ├── certificate.yaml   # TLS certificate
│   │   └── deployment.yaml    # Kubernetes deployment
│   ├── consumer/              # Consumer service resources
│   │   └── deployment.yaml    # Kubernetes deployment
│   ├── dune/                  # Dune service resources
│   │   └── cronjob.yaml       # Kubernetes cronjob
│   ├── publisher/             # Publisher service resources
│   │   └── statefulset.yaml   # Kubernetes statefulset
│   └── webserver/             # Webserver service resources
│       ├── certificate.yaml   # TLS certificate
│       └── deployment.yaml    # Kubernetes deployment
└── tests/                  # Helm chart tests
    ├── api/                # API service tests
    ├── consumer/           # Consumer service tests
    ├── dune/               # Dune service tests
    ├── publisher/          # Publisher service tests
    └── webserver/          # Webserver service tests
```

### Key Files and Their Purpose

#### Chart Configuration Files

- **Chart.yaml**: Defines chart metadata, version, and dependencies
- **values.yaml**: Default configuration values for all services
- **values-local.yaml**: Optimized configuration for local development with reduced resource requirements
- **values-secrets.yaml**: Template for sensitive configuration values

#### Template Files

- **_helpers.tpl**: Contains reusable Go template functions for generating resource names, labels, and other common elements
- **_blocks-*.tpl**: Modular template blocks for generating container specs, pod specs, and other resources
- **_service.yaml**: Template for Kubernetes Service resources
- **_hpa.yaml**: Template for Horizontal Pod Autoscaler resources
- **_pod_monitor.yaml** and **_service_monitor.yaml**: Templates for Prometheus monitoring

#### Service-Specific Templates

Each service has its own directory containing Kubernetes resource definitions:

- **dune/cronjob.yaml**: CronJob for the Dune service that runs on a schedule

#### Dashboard Files

The **dashboards/** directory contains Grafana dashboard configurations for monitoring each service:

- **publisher-metrics.json**: Dashboard for Publisher service metrics
- **consumer-metrics.json**: Dashboard for Consumer service metrics
- **webserver-metrics.json**: Dashboard for Webserver service metrics
- **api-metrics.json**: Dashboard for API service metrics

#### Dependency Charts

The **charts/** directory contains packaged Helm charts for dependencies:

- **nats-1.3.2.tgz**: NATS message broker for event streaming
- **kube-prometheus-stack-27.1.0.tgz**: Prometheus and Grafana for monitoring

## Components

### 1. Publisher Service

The Publisher service connects to a Fuel node, fetches blockchain data (blocks, transactions, receipts, etc.), and publishes it to the NATS message broker.

### 2. Consumer Service

The Consumer service subscribes to the NATS message broker, processes the blockchain data, and stores it in a PostgreSQL database.

### 3. Webserver Service

The Webserver service provides WebSocket endpoints for clients to subscribe to real-time blockchain data.

### 4. API Service

The API service provides REST endpoints for querying blockchain data.

### 5. Dune Service

The Dune service exports blockchain data to S3 for Dune Analytics integration.

### 6. NATS

NATS is used as a message broker for communication between the Publisher and Consumer services.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.2.0+
- PV provisioner support in the underlying infrastructure
- PostgreSQL database (can be deployed separately or using a managed service)

## Dependencies

This chart depends on the following Helm charts:

- NATS (for message streaming)
- Prometheus Stack (for monitoring)

## Installation

### Add the Helm repository

```bash
helm repo add fuel-streams https://fuellabs.github.io/data-systems/charts
helm repo update
```

### Install the chart

```bash
helm install fuel-streams fuel-streams/fuel-streams
```

### Installing with custom values

```bash
helm install fuel-streams fuel-streams/fuel-streams -f values.yaml
```

## Configuration

### Global Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `config.healthChecks` | Enable health checks for all services | `true` |
| `config.prometheus.enabled` | Enable Prometheus monitoring | `true` |
| `commonConfigMap.enabled` | Enable common ConfigMap | `true` |
| `serviceAccount.create` | Create service account | `true` |

### Publisher Service

| Parameter | Description | Default |
|-----------|-------------|---------|
| `publisher.enabled` | Enable Publisher service | `true` |
| `publisher.network` | Network to connect to (mainnet/testnet) | `mainnet` |
| `publisher.image.repository` | Publisher image repository | `ghcr.io/fuellabs/sv-publisher` |
| `publisher.image.tag` | Publisher image tag | `latest` |
| `publisher.config.replicaCount` | Number of replicas | `1` |
| `publisher.storage.size` | Storage size for RocksDB | `500Gi` |
| `publisher.storage.storageClass` | Storage class | `gp3-generic` |

### Consumer Service

| Parameter | Description | Default |
|-----------|-------------|---------|
| `consumer.enabled` | Enable Consumer service | `true` |
| `consumer.network` | Network to connect to (mainnet/testnet) | `mainnet` |
| `consumer.image.repository` | Consumer image repository | `ghcr.io/fuellabs/sv-consumer` |
| `consumer.image.tag` | Consumer image tag | `latest` |
| `consumer.config.replicaCount` | Number of replicas | `3` |

### Webserver Service

| Parameter | Description | Default |
|-----------|-------------|---------|
| `webserver.enabled` | Enable Webserver service | `true` |
| `webserver.network` | Network to connect to (mainnet/testnet) | `mainnet` |
| `webserver.image.repository` | Webserver image repository | `ghcr.io/fuellabs/sv-webserver` |
| `webserver.image.tag` | Webserver image tag | `latest` |
| `webserver.config.replicaCount` | Number of replicas | `3` |
| `webserver.service.host` | Hostname for the service | `stream-staging.fuel.network` |
| `webserver.tls.enabled` | Enable TLS | `true` |
| `webserver.autoscaling.enabled` | Enable autoscaling | `true` |

### API Service

| Parameter | Description | Default |
|-----------|-------------|---------|
| `api.enabled` | Enable API service | `true` |
| `api.network` | Network to connect to (mainnet/testnet) | `mainnet` |
| `api.image.repository` | API image repository | `ghcr.io/fuellabs/sv-api` |
| `api.image.tag` | API image tag | `latest` |
| `api.config.replicaCount` | Number of replicas | `3` |
| `api.service.host` | Hostname for the service | `stream-staging.fuel.network` |
| `api.tls.enabled` | Enable TLS | `true` |
| `api.autoscaling.enabled` | Enable autoscaling | `true` |

### Dune Service

| Parameter | Description | Default |
|-----------|-------------|---------|
| `dune.enabled` | Enable Dune service | `false` |
| `dune.network` | Network to connect to (mainnet/testnet) | `mainnet` |
| `dune.image.repository` | Dune image repository | `ghcr.io/fuellabs/sv-dune` |
| `dune.image.tag` | Dune image tag | `latest` |
| `dune.cronjob.schedule` | Cron schedule | `0 * * * *` |

### NATS Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `nats.enabled` | Enable NATS | `true` |
| `nats.config.cluster.enabled` | Enable NATS clustering | `true` |
| `nats.config.cluster.replicas` | Number of NATS replicas | `5` |
| `nats.config.jetstream.enabled` | Enable JetStream | `true` |

## Local Development

For local development, you can use the `values-local.yaml` file, which configures the services with reduced resource requirements:

```bash
helm install fuel-streams ./cluster/charts/fuel-streams -f ./cluster/charts/fuel-streams/values-local.yaml
```

The local configuration:
- Reduces resource requests and limits
- Uses smaller storage sizes
- Disables TLS
- Uses local image repositories
- Reduces NATS cluster size

## Monitoring

This chart includes Prometheus monitoring by default. The following services expose metrics:

- Publisher: `/metrics` endpoint
- Consumer: `/metrics` endpoint
- Webserver: `/metrics` endpoint
- API: `/metrics` endpoint
- NATS: via the NATS Prometheus exporter

## Troubleshooting

### Common Issues

1. **Publisher can't connect to Fuel node**
   - Check the Fuel node URL in the publisher configuration
   - Verify network connectivity between the publisher pod and the Fuel node

2. **Consumer can't connect to NATS**
   - Check NATS service is running: `kubectl get pods -l app=nats`
   - Verify NATS URL in the consumer configuration

3. **Database connection issues**
   - Verify DATABASE_URL in the common ConfigMap
   - Check database credentials and connectivity

4. **Storage issues**
   - Check if PVCs are bound: `kubectl get pvc`
   - Verify storage class exists and is available

### Useful Commands

```bash
# Check pod status
kubectl get pods -l app.kubernetes.io/instance=fuel-streams

# Check logs for a specific service
kubectl logs -l app.kubernetes.io/name=publisher

# Check NATS status
kubectl exec -it fuel-streams-nats-0 -- nats-top
```

## Advanced Configuration

### High Availability Setup

For production environments, consider:

1. Increasing replica counts:
   ```yaml
   publisher.config.replicaCount: 2
   consumer.config.replicaCount: 5
   webserver.config.replicaCount: 5
   api.config.replicaCount: 5
   ```

2. Enabling autoscaling for all services:
   ```yaml
   consumer.autoscaling.enabled: true
   consumer.autoscaling.minReplicas: 3
   consumer.autoscaling.maxReplicas: 10
   ```

3. Configuring node affinity for distributing pods across zones:
   ```yaml
   config.affinity.podAntiAffinity.requiredDuringSchedulingIgnoredDuringExecution:
     - labelSelector:
         matchLabels:
           app.kubernetes.io/instance: fuel-streams
       topologyKey: topology.kubernetes.io/zone
   ```

### Custom Database Configuration

To use an external database:

```yaml
commonConfigMap:
  data:
    DATABASE_URL: "postgresql://username:password@your-db-host:5432/fuel_streams?sslmode=require"
```

### TLS Configuration

For production environments with custom TLS certificates:

```yaml
webserver.tls:
  enabled: true
  certificate:
    issuer: your-cert-issuer
    duration: 2160h
    renewBefore: 360h
```

## Upgrading

### From 0.10.x to 0.11.x

- The chart now uses StatefulSets for the Publisher service instead of Deployments
- NATS configuration has been updated to support JetStream
- Resource requirements have been adjusted

To upgrade:

```bash
helm upgrade fuel-streams fuel-streams/fuel-streams
```

## License

This chart is licensed under the Apache License 2.0 - see the LICENSE file for details.
