{{/*
Expand the name of the chart.
*/}}
{{- define "fuel-streams.name" -}}
{{- default .Chart.Name .Values.config.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "fuel-streams.fullname" -}}
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
{{- define "fuel-streams.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "fuel-streams.labels" -}}
helm.sh/chart: {{ include "fuel-streams.chart" . }}
{{ include "fuel-streams.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "fuel-streams.selectorLabels" -}}
app.kubernetes.io/name: {{ include "fuel-streams.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "fuel-streams.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "fuel-streams.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Helper function to traverse and retrieve values from nested paths in Values.yaml
Parameters:
  - context: The root context (.)
  - path: Dot-notation string path (e.g., "global.resources.limits")
Returns: The value at the specified path, or empty dict if not found
*/}}
{{- define "get-value-from-path" -}}
{{- $current := .context }}
{{- if .path }}
  {{- range splitList "." .path }}
    {{- if and $current (hasKey $current .) }}
      {{- $current = index $current . }}
    {{- else }}
      {{- $current = dict }}
    {{- end }}
  {{- end }}
{{- end }}
{{- $current }}
{{- end }}

{{/*
Merges configuration values between global defaults and service-specific overrides
Parameters:
  - service: Service name (e.g., "prometheus", "grafana")
  - component: Component being configured (e.g., "resources", "securityContext")
  - context: Root context (.)
  - defaultKey: Path to global default values
  - path: Path to service-specific override values
Returns: Deep-merged YAML configuration with service values taking precedence
*/}}
{{- define "merge-with-defaults" -}}
{{- $defaultConfig := dict }}
{{- $serviceConfig := dict }}
{{- $context := default dict .context }}
{{- $values := default dict $context.Values }}

{{- if and $context $values }}
  {{- $defaultConfig = include "get-value-from-path" (dict "context" $values "path" .defaultKey) | fromYaml }}
{{- end }}

{{- if and .service $values (hasKey $values .service) }}
  {{- $serviceConfig = include "get-value-from-path" (dict "context" (index $values .service) "path" .path) | fromYaml }}
{{- end }}

{{- $mergedConfig := deepCopy $defaultConfig }}
{{- if $serviceConfig }}
{{- $_ := mustMergeOverwrite $mergedConfig $serviceConfig }}
{{- end }}
{{- toYaml $mergedConfig }}
{{- end }}

{{/*
Renders a YAML field with its key name, supporting nested path lookups
Parameters:
  - field: The field name to render (e.g., "nodeSelector", "tolerations")
  - path: Dot-notation path to the field's value in Values.yaml
Example:
  {{- include "render-field-with-key" (dict "field" "nodeSelector" "path" "config.nodeSelector") }}
Returns:
  nodeSelector:
    key: value
*/}}
{{- define "render-field-with-key" -}}
{{- $value := include "get-value-from-path" (dict "context" . "path" .path) | fromYaml }}
{{- if $value }}
{{ $.field }}:
  {{- toYaml $value | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Renders only the YAML value without the field name, supporting nested path lookups
Parameters:
  - field: Field name (used for context only)
  - path: Dot-notation path to the value in Values.yaml
Example:
  {{- include "render-field-value" (dict "field" "labels" "path" "config.labels") }}
Returns:
  key: value
*/}}
{{- define "render-field-value" -}}
{{- $value := include "get-value-from-path" (dict "context" . "path" .path) | fromYaml }}
{{- if $value }}
{{- toYaml $value | nindent 2 }}
{{- end }}
{{- end }}

