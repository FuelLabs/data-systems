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

1. To setup minikube cluster, run the script `./setup_minikube.sh`. To start the cluster, run `./start_minikube.sh` which should spin up the minikube cluster for you. Make sure there are no errors!
2. Run `kubectl create namespace fuel-local` to create a new namespace on the cluster called `fuel-local`
3. Run `kubectl config use-context minikube && kubectl config set-context --current --cluster=minikube --namespace=fuel-local` to set the current context to the latter namespace
4. Run `kubectl config get-contexts` to make sure your cluster is listed as `minikube` and the namespace `fuel-local` belongs to it
5. Run `scripts/traefik2-ds.sh` scrupt.
6. Run `make tilt_up/down/reset` to start/shutdown/reset tilt with the entire stack and all services in it. This needs to be run from the root of the project!

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
