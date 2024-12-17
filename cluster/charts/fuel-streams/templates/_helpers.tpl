{{/*
Expand the name of the chart.
If nameOverride is provided in Values.config, use that instead of .Chart.Name.
The result is truncated to 63 chars and has any trailing "-" removed to comply with Kubernetes naming rules.
Returns: String - The chart name, truncated and cleaned
*/}}
{{- define "fuel-streams.name" -}}
{{- default .Chart.Name .Values.config.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
This template follows these rules:
1. If fullnameOverride is set in Values.config, use that directly
2. Otherwise, combine the release name with the chart name:
   - If the release name already contains the chart name, just use the release name
   - If not, concatenate release name and chart name with a hyphen
The result is truncated to 63 chars and has any trailing "-" removed to comply with Kubernetes naming rules.
Returns: String - The fully qualified app name, truncated and cleaned
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
Combines the chart name and version with a hyphen, replaces "+" with "_",
and truncates to 63 chars removing any trailing "-".
Returns: String - The chart name and version formatted for use as a label
*/}}
{{- define "fuel-streams.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
Provides a consistent set of labels that should be applied to all resources in the chart.
Includes:
- Helm chart metadata
- Selector labels (app name and instance)
- App version (if defined)
- Managed-by label indicating Helm management
Parameters:
  - name: Optional custom name to use instead of the default name
  - .: Full context (passed automatically or as "context")
Returns: Map - A set of key-value pairs representing Kubernetes labels
Example:
  {{- include "fuel-streams.labels" . }}
  # Or with custom name:
  {{- include "fuel-streams.labels" (dict "name" "custom-name" "context" $) }}
*/}}
{{- define "fuel-streams.labels" -}}
{{- $context := default . .context -}}
helm.sh/chart: {{ include "fuel-streams.chart" $context }}
{{ include "fuel-streams.selectorLabels" (dict "name" .name "context" $context) }}
{{- if $context.Chart.AppVersion }}
app.kubernetes.io/version: {{ $context.Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ $context.Release.Service }}
{{- end }}

{{/*
Selector labels
Core identifying labels used for object selection and service discovery.
These labels should be used consistently across all related resources.
Parameters:
  - name: Optional custom name to use instead of the default name
  - .: Full context (passed automatically or as "context")
Returns: Map - A set of key-value pairs for Kubernetes selector labels
Example:
  {{- include "fuel-streams.selectorLabels" . }}
  # Or with custom name:
  {{- include "fuel-streams.selectorLabels" (dict "name" "custom-name" "context" $) }}
*/}}
{{- define "fuel-streams.selectorLabels" -}}
{{- $context := default . .context -}}
{{- $name := default (include "fuel-streams.name" $context) .name -}}
app.kubernetes.io/name: {{ $name }}
app.kubernetes.io/instance: {{ $context.Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
Logic:
- If service account creation is enabled, use the fullname template with "-service-account" suffix
Returns: String - The name of the service account to use
*/}}
{{- define "fuel-streams.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- printf "%s-%s" (include "fuel-streams.fullname" .) "service-account" }}
{{- end }}
{{- end }}

{{/*
Merges configuration values between global defaults and service-specific overrides
Parameters:
  - context: Root context for accessing global values
  - service: Service name for service-specific overrides
  - defaultKey: Key for default configuration values
  - path: Path to service-specific configuration values
Example:
  {{- include "merge" (dict "context" . "service" "publisher" "defaultKey" "livenessProbe" "path" "config.livenessProbe") }}
Returns: Deep-merged YAML configuration with service values taking precedence
*/}}
{{- define "merge" -}}
{{- $defaultConfig := dict }}
{{- $serviceConfig := dict }}
{{- $context := .context }}
{{- $service := .service }}
{{- $defaultKey := .defaultKey }}
{{- $path := .path }}

{{- if and $context $context.Values }}
  {{- $defaultConfig = include "get-value-from-path" (dict "context" $context.Values "path" $defaultKey) | fromYaml }}
{{- end }}

{{- if and $context $context.Values }}
  {{- $serviceConfig = include "get-value-from-path" (dict "context" (index $context.Values $service) "path" $path) | fromYaml }}
{{- end }}

{{- $mergedConfig := deepCopy $defaultConfig }}
{{- if and $serviceConfig (not (empty $serviceConfig)) }}
  {{- $_ := mustMergeOverwrite $mergedConfig $serviceConfig }}
{{- end }}
{{- toYaml $mergedConfig }}
{{- end }}

{{/*
Get value from nested path with improved error handling
Parameters:
  - context: The context object to traverse
  - path: Dot-notation string path
Returns: Value at path or empty if not found
*/}}
{{- define "get-value-from-path" -}}
  {{- $current := .context }}
  {{- if and .path (not (empty .path)) }}
    {{- range $part := splitList "." .path }}
      {{- if and $current (kindIs "map" $current) }}
        {{- if hasKey $current $part }}
          {{- $current = index $current $part }}
        {{- else }}
          {{- $current = dict }}
        {{- end }}
      {{- else }}
        {{- $current = dict }}
      {{- end }}
    {{- end }}
  {{- end }}
  {{- if $current }}
    {{- toYaml $current }}
  {{- end }}
{{- end }}

{{/*
Set field and value with improved validation
Parameters:
  - field: Field name to set
  - path: Path to value
  - context: Context object
Returns: Field and value if value exists and is not empty
*/}}
{{- define "set-field-and-value" -}}
{{- $value := include "get-value-from-path" . | fromYaml }}
{{- if and $value (not (empty $value)) (not (eq (kindOf $value) "invalid")) }}
{{ .field }}:
  {{- toYaml $value | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Set value only with improved validation
Parameters:
  - path: Path to value
  - context: Context object
Returns: Value if it exists and is not empty
*/}}
{{- define "set-value" -}}
{{- $value := include "get-value-from-path" . | fromYaml }}
{{- if and $value (not (empty $value)) (not (eq (kindOf $value) "invalid")) }}
  {{- toYaml $value | nindent 0 }}
{{- end }}
{{- end }}
