{{/*
Define Kubernetes pod-level security context settings.

Parameters:
  - context: The context (.)
  - service: The service (.)

Example usage in values.yaml:
  securityContext:  # Default values
    fsGroup: 1000
    runAsNonRoot: true
    seccompProfile:
      type: RuntimeDefault

  config:  # User-provided values
    securityContext:
      fsGroup: 2000
      runAsNonRoot: true

Returns:
  securityContext:  # Deep merged result
    fsGroup: 2000  # From config, overrides default
    runAsNonRoot: true  # From both
    seccompProfile:  # From default
      type: RuntimeDefault
*/}}
{{- define "k8s.security-context" -}}
securityContext:
{{- include "merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "securityContext" "path" "config.securityContext") | nindent 2 }}
{{- end }}

{{/*
Define Kubernetes container-level security context settings.

Parameters:
  - context: The context (.)
  - service: The service (.)

Example usage in values.yaml:
  containerSecurityContext:
    runAsUser: 1000
    runAsGroup: 1000
    allowPrivilegeEscalation: false
    readOnlyRootFilesystem: true
  config:
    containerSecurityContext:
      runAsUser: 2000

Returns:
  securityContext:
    runAsUser: 2000  # From config, overrides default
    runAsGroup: 1000 # From default
    allowPrivilegeEscalation: false # From default
    readOnlyRootFilesystem: true # From default
*/}}
{{- define "k8s.container-security-context" -}}
securityContext:
{{- include "merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "containerSecurityContext" "path" "config.containerSecurityContext") | nindent 2 }}
{{- end }}

{{/*
Configure all Kubernetes probe settings (liveness, readiness, and startup) for a pod.

Parameters:
  - context: The context (.)
  - service: The service name

Example usage in values.yaml:
  # Default values
  probes:
    path: /health
    port: http
    scheme: HTTP
    liveness:
      enabled: true
      initialDelaySeconds: 10
    readiness:
      enabled: true
    startup:
      enabled: false

  # Service-specific values
  myservice:
    config:
      probes:
        path: /custom-health
        liveness:
          initialDelaySeconds: 20
*/}}
{{- define "pod.probes" -}}
{{- $defaultConfig := include "merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "probes" "path" "probes") | fromYaml }}
{{- if $defaultConfig }}
{{- with $defaultConfig }}
{{- if .liveness.enabled }}
livenessProbe:
  httpGet:
    path: {{ .path }}
    port: {{ .port }}
    scheme: {{ .scheme | default "HTTP" | upper }}
  {{- with .liveness }}
  {{- omit . "enabled" | toYaml | nindent 2 }}
  {{- end }}
{{- end }}
{{- if .readiness.enabled }}
readinessProbe:
  httpGet:
    path: {{ .path }}
    port: {{ .port }}
    scheme: {{ .scheme | default "HTTP" | upper }}
  {{- with .readiness }}
  {{- omit . "enabled" | toYaml | nindent 2 }}
  {{- end }}
{{- end }}
{{- if .startup.enabled }}
startupProbe:
  httpGet:
    path: {{ .path }}
    port: {{ .port }}
    scheme: {{ .scheme | default "HTTP" | upper }}
  {{- with .startup }}
  {{- omit . "enabled" | toYaml | nindent 2 }}
  {{- end }}
{{- end }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Configure basic Kubernetes resource metadata fields.
Parameters:
  - context: The context (.)
  - suffix: Optional suffix to append to resource name
  - name: Optional name override
*/}}
{{- define "k8s.metadata" -}}
name: {{ default (include "fuel-streams.fullname" .context) .name }}{{ .suffix }}
namespace: {{ .context.Release.Namespace }}
{{- end }}

