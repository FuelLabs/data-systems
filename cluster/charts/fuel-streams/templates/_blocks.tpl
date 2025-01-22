{{/*
Configure nats accounts
*/}}
{{- define "nats-accounts" -}}
data:
  auth.conf: |
    accounts {
      SYS: {
        users: [
          {user: $NATS_SYSTEM_USER, password: $NATS_SYSTEM_PASS}
        ]
      }
      ADMIN: {
        jetstream: enabled
        users: [
          {user: $NATS_ADMIN_USER, password: $NATS_ADMIN_PASS}
        ]
      }
    }
{{- end }}

{{- define "k8s.default-affinity" -}}
podAntiAffinity:
  preferredDuringSchedulingIgnoredDuringExecution:
    - weight: 100
      podAffinityTerm:
        labelSelector:
          matchLabels:
            app.kubernetes.io/component: publisher
        topologyKey: topology.kubernetes.io/zone
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
app: {{ .context.Chart.Name }}
{{- end }}

{{/*
Configure resource header including replicas and selector labels
*/}}
{{- define "k8s.resource-metadata" -}}
{{- $suffix := printf "-%s" .name -}}
{{- $component := .component | default .name }}
{{- include "k8s.metadata" (dict "context" .root "suffix" $suffix) }}
labels:
  {{- include "fuel-streams.labels" (dict "name" .name "context" .root) | nindent 2 }}
  {{- include "set-value" (dict "root" .root "context" .context "path" "config.labels") | nindent 2 -}}
  app.kubernetes.io/component: {{ $component }}
{{- if not .noAnnotations -}}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "annotations" "path" "config.annotations") }}
{{- end }}
{{- end }}

{{/*
Configure resource annotations
*/}}
{{- define "k8s.resource-annotations" -}}
{{- include "set-value" (dict "root" .root "context" .context "path" "config.annotations") }}
{{- end }}

{{/*
Configure pod spec header including replicas and selector labels
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
  - name: Name of the service for selector labels
Returns: YAML configuration for pod spec header
*/}}
{{- define "k8s.pod-spec-common" -}}
{{- if not .context.autoscaling.enabled }}
replicas: {{ .context.config.replicaCount }}
{{- end }}
selector:
  matchLabels:
    {{- include "fuel-streams.selectorLabels" (dict "name" .name "context" .root) | nindent 4 }}
{{- end }}

{{/*
Configure pod template metadata including annotations and labels
Parameters:
  - root: Root context object for fallback values 
  - context: Service-specific context object containing configuration
  - name: Name of the service for labels
Returns: YAML configuration for pod template metadata
*/}}
{{- define "k8s.template-labels" -}}
{{- $component := .component | default .name }}
{{- include "fuel-streams.labels" (dict "name" .name "context" .root) }}
{{- include "set-value" (dict "root" .root "context" .context "path" "config.labels") }}
app.kubernetes.io/component: {{ $component }}
{{- end }}

{{/*
Configure pod template metadata including annotations and labels
Parameters:
  - root: Root context object for fallback values 
  - context: Service-specific context object containing configuration
  - name: Name of the service for labels
Returns: YAML configuration for pod template metadata
*/}}
{{- define "k8s.template-annotations" -}}
{{- include "set-value" (dict "root" .root "context" .context "path" "config.podAnnotations") }}
{{- end }}

{{- define "k8s.template-metadata" -}}
metadata:
  {{- include "set-field-and-value" (dict "root" .root "context" .context "field" "annotations" "path" "config.podAnnotations") | nindent 4 }}
  labels:
    {{- include "k8s.template-labels" (dict "root" .root "context" .context) | nindent 4 }}
{{- end }}

{{/*
Configure pod-level settings including security, scheduling and image pull configuration
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: YAML configuration for pod-level settings
*/}}
{{- define "k8s.pod-config" -}}
{{- if .root.Values.serviceAccount.create }}
serviceAccountName: {{ include "fuel-streams.serviceAccountName" .root }}
{{- end }}

{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "imagePullSecrets") }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "nodeSelector") }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "affinity") }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "tolerations") }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "securityContext" "path" "config.podSecurityContext") }}
{{- end }}

{{/*
Configure container-level settings including resource requests, security context, and probes
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: YAML configuration for container-level settings
*/}}
{{- define "k8s.container-config" -}}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "resources") }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "securityContext" "path" "config.containerSecurityContext") }}

{{- if .root.Values.config.healthChecks }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "livenessProbe") }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "readinessProbe") }}
{{- include "set-field-and-value" (dict "root" .root "context" .context "field" "startupProbe") }}
{{- end }}

ports:
  - name: server
    containerPort: {{ .context.port }}
    protocol: TCP
  {{- with .context.ports }}
  {{- toYaml . | nindent 2 }}
  {{- end }}

env:
  {{- if .context.port }}
  - name: PORT
    value: {{ .context.port | quote }}
  {{- end }}
  {{- range $key, $value := .context.env }}
  - name: {{ $key }}
    value: {{ $value | quote }}
  {{- end }}

envFrom:
- configMapRef:
    name: {{ include "fuel-streams.fullname" .root }}-config
    optional: true
- secretRef:
    name: {{ include "fuel-streams.fullname" .root }}-keys
    optional: true
{{- with .context.envFrom }}
{{- toYaml . | nindent 0 }}
{{- end }}
{{- end }}
