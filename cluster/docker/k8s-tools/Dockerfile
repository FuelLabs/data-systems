ARG BASE_DOCKER_VERSION=20.10.6
FROM docker:${BASE_DOCKER_VERSION}

RUN apk add --update --no-cache \
  bash \
  libffi-dev \
  openssl-dev \
  libgcc \
  curl \
  wget \
  bind-tools \
  jq \
  git \
  git-lfs \
  openssh \
  openssh-client \
  gcc \
  rust \
  cargo \
  gettext \
  libc-dev \
  python3-dev \
  py3-pip \
  rsync \
  make \
  cmake \
  moreutils \
  busybox-extras \
  vim \
  ca-certificates \
  netcat-openbsd \
  patch

ARG HELM=3.5.4
ARG KUBECTL=1.19.9
ARG KIND=0.10.0
ARG YQ=4.9.1
ARG RANCHER=2.4.11
ARG DOCKER_COMPOSE=1.29.2
ARG AWS_IAM_AUTH=0.5.3

RUN \
  curl -L https://get.helm.sh/helm-v${HELM}-linux-amd64.tar.gz | tar xvz && \
  curl -L https://github.com/rancher/cli/releases/download/v${RANCHER}/rancher-linux-amd64-v${RANCHER}.tar.gz | tar xvz && \
  curl -Lo kubectl https://storage.googleapis.com/kubernetes-release/release/v${KUBECTL}/bin/linux/amd64/kubectl && \
  curl -Lo kind https://github.com/kubernetes-sigs/kind/releases/download/v${KIND}/kind-linux-amd64 && \
  curl -Lo yq https://github.com/mikefarah/yq/releases/download/v${YQ}/yq_linux_amd64 && \
  curl -Lo aws-iam-authenticator https://github.com/kubernetes-sigs/aws-iam-authenticator/releases/download/v${AWS_IAM_AUTH}/aws-iam-authenticator_${AWS_IAM_AUTH}_linux_amd64 && \
  mv linux-amd64/helm /usr/local/bin/ && \
  mv rancher-v${RANCHER}/rancher /usr/local/bin/ && \
  mv kubectl /usr/local/bin/ && \
  mv kind /usr/local/bin/ && \
  mv yq /usr/local/bin/ && \
  mv aws-iam-authenticator /usr/local/bin/ && \
  chmod +x /usr/local/bin/helm /usr/local/bin/rancher /usr/local/bin/kubectl /usr/local/bin/kind && \
  chmod +x /usr/local/bin/yq /usr/local/bin/aws-iam-authenticator && \
  rm -rf linux-amd64 rancher-v${RANCHER}

RUN pip3 install --upgrade pip setuptools && \
  pip3 install docker-compose==${DOCKER_COMPOSE} requests beautifulsoup4
