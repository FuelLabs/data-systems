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
app: {{ .context.Chart.Name }}
{{- end }}

{{/*
Configure pod security context settings by merging global and service-specific values.
Parameters:
  - context: Root context for accessing global values
  - service: Service name for service-specific overrides
Returns: Security context configuration for pod-level settings
Example:
  {{- include "k8s.security-context" (dict "context" . "service" "publisher") }}
*/}}
{{- define "k8s.security-context" -}}
securityContext:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "securityContext" "path" "config.securityContext") | nindent 4 }}
{{- end }}

{{/*
Configure container security context settings by merging global and service-specific values.
Parameters:
  - context: Root context for accessing global values
  - service: Service name for service-specific overrides
Returns: Security context configuration for container-level settings
Example:
  {{- include "k8s.container-security-context" (dict "context" . "service" "publisher") }}
*/}}
{{- define "k8s.container-security-context" -}}
securityContext:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "containerSecurityContext" "path" "config.containerSecurityContext") | nindent 4 }}
{{- end }}

{{/*
Configure container probe settings by merging global and service-specific values.
Parameters:
  - context: Root context for accessing global values
  - service: Service name for service-specific overrides
Returns: Probe configurations for liveness, readiness, and startup
Example:
  {{- include "k8s.probes" (dict "context" . "service" "publisher") }}
*/}}
{{- define "k8s.probes" -}}
{{- if .context.Values.config.healthChecks }}
livenessProbe:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "livenessProbe" "path" "config.livenessProbe") | nindent 2 }}
readinessProbe:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "readinessProbe" "path" "config.readinessProbe") | nindent 2 }}
startupProbe:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "startupProbe" "path" "config.startupProbe") | nindent 2 }}
{{- end }}
{{- end }}