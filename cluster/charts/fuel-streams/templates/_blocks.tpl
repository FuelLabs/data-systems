{{/*
Define Kubernetes pod-level security context settings.

This helper template configures security context settings at the pod level by merging user-provided
configurations with default values. Pod security context controls pod-wide security attributes like:

- File system group settings (fsGroup)
- Host namespace sharing (hostNetwork, hostPID, hostIPC) 
- SELinux context
- Sysctls
- Seccomp profiles

Example usage in values.yaml:
  config:
    securityContext:
      fsGroup: 1000
      runAsNonRoot: true
      seccompProfile:
        type: RuntimeDefault
*/}}
{{- define "k8s.security-context" -}}
securityContext:
{{- include "merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "securityContext" "path" "config.securityContext") | nindent 2 }}
{{- end }}

{{/*
Define Kubernetes container-level security context settings.

This helper template configures security context settings at the container level by merging user-provided 
configurations with default values. Container security context controls container-specific security 
attributes like:

- User/group ID settings (runAsUser, runAsGroup)
- Linux capabilities
- SELinux context
- Privilege settings (privileged, allowPrivilegeEscalation)
- Read-only root filesystem
- Seccomp/AppArmor profiles

Example usage in values.yaml:
  config:
    containerSecurityContext:
      runAsUser: 1000
      runAsGroup: 1000
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
*/}}
{{- define "k8s.container-security-context" -}}
securityContext:
{{- include "merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "containerSecurityContext" "path" "config.containerSecurityContext") | nindent 2 }}
{{- end }}

{{/*
Configure all Kubernetes probe settings (liveness, readiness, and startup) for a pod.

This helper template consolidates the configuration of all three probe types (liveness, readiness, 
and startup) and merges them with default values. It provides a single entry point for configuring
pod health checks.

The template:
1. Merges user-provided probe configurations with defaults using merge-with-defaults
2. Creates a probe configuration object with the merged settings
3. Includes individual probe templates for liveness, readiness and startup checks

The probe settings control how Kubernetes monitors pod health:
- Liveness probes check if the container is running properly
- Readiness probes verify if the pod can receive traffic
- Startup probes allow for longer initialization time on container startup

Example usage in values.yaml:
  probes:
    path: /health
    port: http
    scheme: HTTP
    liveness:
      enabled: true
      initialDelaySeconds: 10
      periodSeconds: 10
      timeoutSeconds: 1
      failureThreshold: 3
      successThreshold: 1
    readiness:
      enabled: true
      initialDelaySeconds: 5
      periodSeconds: 10
      timeoutSeconds: 1
      failureThreshold: 3
      successThreshold: 1
    startup:
      enabled: true
      initialDelaySeconds: 30
      periodSeconds: 10
      timeoutSeconds: 1
      failureThreshold: 30
      successThreshold: 1
*/}}
{{- define "pod.probes" -}}
{{- $mergedConfig := include "merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "probes" "path" "probes") | fromYaml }}
{{- $probeConfig := dict "mergedProbes" $mergedConfig "serviceProbes" .serviceProbes "context" .context }}
{{- include "pod.liveness-probe" $probeConfig }}
{{- include "pod.readiness-probe" $probeConfig }}
{{- include "pod.startup-probe" $probeConfig }}
{{- end }}

{{/*
Configure HTTP liveness probe settings for a pod.

This helper template configures liveness probe settings for Kubernetes pods using HTTP checks.
Liveness probes determine if a container is healthy and running as expected. If a liveness
probe fails, Kubernetes will restart the container to try to restore service.
*/}}
{{- define "pod.liveness-probe" -}}
{{- if and .mergedProbes.liveness .mergedProbes.liveness.enabled }}
livenessProbe:
  httpGet:
    path: {{ .serviceProbes.path }}
    port: {{ .serviceProbes.port }}
    scheme: {{ .serviceProbes.scheme | default "HTTP" | upper }}
  {{- with .mergedProbes.liveness }}
  initialDelaySeconds: {{ .initialDelaySeconds }}
  periodSeconds: {{ .periodSeconds }}
  timeoutSeconds: {{ .timeoutSeconds }}
  failureThreshold: {{ .failureThreshold }}
  successThreshold: {{ .successThreshold }}
  {{- end }}
{{- end }}
{{- end }}

{{/*
Configure HTTP readiness probe settings for a pod.

This helper template configures readiness probe settings for Kubernetes pods using HTTP checks.
Readiness probes determine if a container is ready to serve traffic. A pod is considered ready
when all of its containers are ready. Services will only send traffic to pods that are ready.
*/}}
{{- define "pod.readiness-probe" -}}
{{- if and .mergedProbes.readiness .mergedProbes.readiness.enabled }}
readinessProbe:
  httpGet:
    path: {{ .serviceProbes.path }}
    port: {{ .serviceProbes.port }}
    scheme: {{ .serviceProbes.scheme | default "HTTP" | upper }}
  {{- with .mergedProbes.readiness }}
  initialDelaySeconds: {{ .initialDelaySeconds }}
  periodSeconds: {{ .periodSeconds }}
  timeoutSeconds: {{ .timeoutSeconds }}
  failureThreshold: {{ .failureThreshold }}
  successThreshold: {{ .successThreshold }}
  {{- end }}
{{- end }}
{{- end }}

{{/*
Configure HTTP startup probe settings for a pod.

This helper template configures startup probe settings for Kubernetes pods using HTTP checks.
Startup probes help determine when a container application has started and is ready to serve traffic.
They are particularly useful for slow-starting containers to prevent them from being killed
before they are fully initialized.
*/}}
{{- define "pod.startup-probe" -}}
{{- if and .mergedProbes.startup .mergedProbes.startup.enabled }}
startupProbe:
  httpGet:
    path: {{ .serviceProbes.path }}
    port: {{ .serviceProbes.port }}
    scheme: {{ .serviceProbes.scheme | default "HTTP" | upper }}
  {{- with .mergedProbes.startup }}
  initialDelaySeconds: {{ .initialDelaySeconds }}
  periodSeconds: {{ .periodSeconds }}
  timeoutSeconds: {{ .timeoutSeconds }}
  failureThreshold: {{ .failureThreshold }}
  successThreshold: {{ .successThreshold }}
  {{- end }}
{{- end }}
{{- end }}

{{/*
Define standard Kubernetes resource metadata.
Generates consistent name, namespace, labels and annotations for Kubernetes resources.

Parameters:
  - context: The root chart context (.)
  - service: The service name to append to the resource name (e.g. "prometheus", "grafana") 
  - labels: Optional additional labels to merge with common labels
  - annotations: Optional annotations to add to the resource

Example:
  {{- include "resource.metadata" (dict "service" "prometheus" "context" . "labels" (dict "custom" "value") "annotations" (dict "prometheus.io/scrape" "true")) }}

Returns:
  name: release-name-prometheus
  namespace: default
  labels:
    app.kubernetes.io/name: chart-name
    app.kubernetes.io/instance: release-name
    custom: value
  annotations:
    prometheus.io/scrape: "true"
*/}}
{{- define "resource.metadata" -}}
name: {{ include "fuel-streams.fullname" .context }}-{{ .service }}
namespace: {{ .context.Release.Namespace }}
labels:
  {{- include "fuel-streams.labels" .context | nindent 4 }}
  {{- if .labels }}
  {{- toYaml .labels | nindent 4 }}
  {{- end }}
{{- if .annotations }}
annotations:
  {{- toYaml .annotations | nindent 4 }}
{{- end }}
{{- end }}

{{/*
Renders a YAML field with its key name, supporting nested path lookups
Parameters:
  - field: The field name to render (e.g., "nodeSelector", "tolerations")
  - path: Dot-notation path to the field's value in Values.yaml
  - context: The context to use (e.g., $publisher)
Example:
  {{- include "render-field-with-key" (dict "field" "nodeSelector" "path" "config.nodeSelector" "context" $publisher) }}
Returns:
  nodeSelector:
    key: value
*/}}
{{- define "render-field-with-key" -}}
{{- $value := include "get-value-from-path" . | fromYaml }}
{{- if and $value (not (empty $value)) }}
{{ .field }}:
{{- toYaml $value | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Renders only the YAML value without the field name, supporting nested path lookups
Parameters:
  - field: Field name (used for context only)
  - path: Dot-notation path to the value in Values.yaml
  - context: The context to use (e.g., $publisher)
Example:
  {{- include "render-field-value" (dict "field" "labels" "path" "config.labels" "context" $publisher) }}
Returns:
  key: value
*/}}
{{- define "render-field-value" -}}
{{- $value := include "get-value-from-path" . | fromYaml }}
{{- if and $value (not (empty $value)) }}
{{- toYaml $value | nindent 0 }}
{{- end }}
{{- end }}

{{/*
Helper function to traverse and retrieve values from nested paths
Parameters:
  - context: The context object to traverse
  - path: Dot-notation string path (e.g., "config.resources.limits")
Returns: The value at the specified path, or empty if not found
*/}}
{{- define "get-value-from-path" -}}
{{- $current := .context }}
{{- range $part := splitList "." .path }}
{{- if $current }}
{{- $current = index $current $part }}
{{- end }}
{{- end }}
{{- if $current }}
{{- toYaml $current }}
{{- end }}
{{- end }}