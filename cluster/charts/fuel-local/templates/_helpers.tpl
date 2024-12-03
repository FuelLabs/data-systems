{{/*
Expand the name of the chart.
*/}}
{{- define "fuel-local.name" -}}
{{- default .Chart.Name .Values.config.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "fuel-local.fullname" -}}
{{- if .Values.config.fullnameOverride }}
{{- .Values.config.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.config.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "fuel-local.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "fuel-local.labels" -}}
helm.sh/chart: {{ include "fuel-local.chart" . }}
{{ include "fuel-local.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "fuel-local.selectorLabels" -}}
app.kubernetes.io/name: {{ include "fuel-local.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "fuel-local.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "fuel-local.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Generic configuration merger that handles deep merging of configs
Input expects:
- service: name of the service (e.g., "prometheus", "kibana")
- component: name of the component to merge (e.g., "resources", "securityContext")
- context: the root context (.)
- defaultKey: the key for default values (e.g., "global.resources", "global.securityContext")
- path: the path to service-specific config (e.g., "securityContext", "containerSecurityContext")
*/}}
{{- define "common.merge-with-defaults" -}}
{{- $defaultConfig := dict }}
{{- if and .context .context.Values }}
  {{- $parts := splitList "." .defaultKey }}
  {{- $current := .context.Values }}
  {{- range $parts }}
    {{- if hasKey $current . }}
      {{- $current = index $current . }}
    {{- end }}
  {{- end }}
  {{- $defaultConfig = $current }}
{{- end }}
{{- $serviceConfig := dict }}
{{- if and .context .context.Values (hasKey .context.Values .service) (hasKey (index .context.Values .service) .path) }}
{{- $serviceConfig = index (index .context.Values .service) .path }}
{{- end }}
{{- $mergedConfig := deepCopy $defaultConfig }}
{{- if $serviceConfig }}
{{- $_ := mergeOverwrite $mergedConfig $serviceConfig }}
{{- end }}
{{- toYaml $mergedConfig }}
{{- end }}

{{/*
Common resource management
*/}}
{{- define "common.resources" -}}
{{- if not .context.Values.config.disableResourceLimits }}
resources:
{{- include "common.merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "global.resources" "path" "resources") | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Common security context for pods
*/}}
{{- define "common.security-context" -}}
{{- if not .context.Values.config.disableSecurityContext }}
securityContext:
{{- include "common.merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "global.securityContext" "path" "securityContext") | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Common security context for containers
*/}}
{{- define "common.container-security-context" -}}
{{- if not .context.Values.config.disableSecurityContext }}
securityContext:
{{- include "common.merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "global.containerSecurityContext" "path" "containerSecurityContext") | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Common pod anti-affinity for stateful services
*/}}
{{- define "common.pod-anti-affinity" -}}
affinity:
{{- include "common.merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "global.affinity" "path" "affinity") | nindent 2 }}
{{- end }}

{{/*
Common probes configuration
*/}}
{{- define "common.probes" -}}
{{- $mergedConfig := include "common.merge-with-defaults" (dict "service" .service "context" .context "defaultKey" "global.probes" "path" "probes") | fromYaml }}
{{- $probeConfig := dict "mergedProbes" $mergedConfig "serviceProbes" .serviceProbes "context" .context }}
{{- include "common.liveness-probe" $probeConfig }}
{{- include "common.readiness-probe" $probeConfig }}
{{- include "common.startup-probe" $probeConfig }}
{{- end }}

{{/*
Common liveness probe configuration
*/}}
{{- define "common.liveness-probe" -}}
{{- if and .context.Values.config.healthChecks .mergedProbes.liveness .mergedProbes.liveness.enabled }}
livenessProbe:
  httpGet:
    path: {{ .serviceProbes.healthPath }}
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
Common readiness probe configuration
*/}}
{{- define "common.readiness-probe" -}}
{{- if and .context.Values.config.healthChecks .mergedProbes.readiness .mergedProbes.readiness.enabled }}
readinessProbe:
  httpGet:
    path: {{ .serviceProbes.healthPath }}
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
Common startup probe configuration
*/}}
{{- define "common.startup-probe" -}}
{{- if and .context.Values.config.healthChecks .mergedProbes.startup .mergedProbes.startup.enabled }}
startupProbe:
  httpGet:
    path: {{ .serviceProbes.healthPath }}
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
Common metadata for all resources
*/}}
{{- define "common.metadata" -}}
name: {{ include "fuel-local.fullname" .context }}-{{ .service }}
namespace: {{ .context.Values.config.namespace }}
labels:
  {{- include "fuel-local.labels" .context | nindent 4 }}
{{- end }}
