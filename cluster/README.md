# Local Fuel Cluster

A dockerized fuel-data-systems for Kubernetes.

This repo contains instructions how to spin up a full local kubernetes cluster with a fuel-publisher service and all supporting infrastructure.

The latter is intended for local development, but it also allows us to deploy the Helm charts to a cloud-based infra with ease.

## Requirements

The following are prerequisites for spinning up the fuel-data-systems cluster locally:

-   kubectl
    `https://www.howtoforge.com/how-to-install-kubernetes-with-minikube-ubuntu-20-04/`

-   Tilt:
    `https://docs.tilt.dev/install.html`

-   minikube based on the following description:
    `https://phoenixnap.com/kb/install-minikube-on-ubuntu`
    `https://minikube.sigs.k8s.io/docs/start/`

...or alternatively use this tool which will automatically set up your cluster:
`https://github.com/tilt-dev/ctlptl##minikube-with-a-built-in-registry`

## Setup

1. To setup and start the local environment, run:
   ```bash
   make cluster_setup  # Sets up both minikube and kubernetes configuration
   ```

   Alternatively, you can run the setup steps individually:
   ```bash
   make minikube_setup  # Sets up minikube with required addons
   make k8s_setup       # Configures kubernetes with proper namespace and context
   ```

   You can also start the minikube cluster without running the setup script:
   ```bash
   make minikube_start  # Start minikube cluster
   ```

2. Start the Tilt services:
   ```bash
   make cluster_up  # Starts Tiltfile services
   ```

You can use the following commands to manage the services:
```bash
make cluster_up     # Start services
make cluster_down   # Stop services
make cluster_reset  # Reset services
make minikube_start # Start minikube (if you've already run setup before)
```

## Using `k9s` for an interactive terminal UI

Install k9s from [here](https://github.com/derailed/k9s)

Run it with `k9s --context=<your kubectl context> --namespace=<namespace you want to watch>` e.g. `k9s --context=minikube --namespace=fuel-local`. You can do things like view logs with `l`, describe with `d`, delete with `Ctrl+d`.

## Useful links

-   How [kubernetes works](https://www.youtube.com/watch?v=ZuIQurh_kDk)
-   Kubernetes [concepts](https://kubernetes.io/docs/concepts/)
-   Kubectl [overview](https://kubernetes.io/docs/reference/kubectl/overview/)
-   Kubectl [cheat sheet](https://kubernetes.io/docs/reference/kubectl/cheatsheet/)
-   Helm [chart tutorial](https://docs.bitnami.com/kubernetes/how-to/create-your-first-helm-chart/), then examine the helm charts in this repository, and the values yaml files that are used to template them. The defults values are in the charts themselves as `values.yaml`, and the values for specific configurations are at `values/<name>.yaml`.
-   Tilt [tutorial](https://docs.tilt.dev/tutorial.html)
