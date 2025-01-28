{{/*
Expand the name of the chart.
If nameOverride is provided in Values.config, use that instead of .Chart.Name.
The result is truncated to 63 chars and has any trailing "-" removed to comply with Kubernetes naming rules.
Returns: String - The chart name, truncated and cleaned
Example:
  Given:
    .Chart.Name = "fuel-streams"
    .Values.config.nameOverride = "custom-name"
  Result: "custom-name"

  Given:
    .Chart.Name = "fuel-streams"
    .Values.config.nameOverride = null
  Result: "fuel-streams"
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
Example:
  Given:
    .Values.config.fullnameOverride = "override-name"
  Result: "override-name"

  Given:
    .Release.Name = "my-release"
    .Chart.Name = "fuel-streams"
    .Values.config.nameOverride = null
    .Values.config.fullnameOverride = null
  Result: "my-release-fuel-streams"

  Given:
    .Release.Name = "fuel-streams-prod"
    .Chart.Name = "fuel-streams"
    .Values.config.nameOverride = null
    .Values.config.fullnameOverride = null
  Result: "fuel-streams-prod"
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
Get a value from a nested path in a context object

Parameters:
  - context: The context object to traverse (required)
  - path: Dot-notation string path to the desired value (required)
    Example: "config.labels" will look for context.config.labels

Returns:
  - The value at the specified path if found
  - Empty dict if path is invalid or value not found
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
Get context value with fallback to root values

Parameters:
  - context: Value-specific context object (required)
    The immediate context containing the desired value (e.g., service-specific values)
  - root: Root context for fallback values (required)
    The root context containing global/default values (typically . or .Values)
  - path: Dot-notation string path to the desired value (required)
    The path to the value in dot notation (e.g., "config.labels")
  - rootPath: Optional alternative path to check in root context
    If provided, this path will be used instead of 'path' when checking root context

Returns:
  - The value from context if found and valid
  - The value from root (using rootPath if provided, otherwise path) if not found in context
  - Empty string if value is not found in either context or root

Example Usage:
1. With same path in context and root:
   {{- include "get-context-value" (dict "context" $publisherValues "root" . "path" "config.labels") }}

2. With different root path:
   {{- include "get-context-value" (dict "context" $publisherValues "root" . "path" "config.autoscaling" "rootPath" "config.defaultAutoscaling") }}
*/}}
{{- define "get-context-value" -}}
{{- $contextConfig := dict }}
{{- $rootConfig := dict }}
{{- $context := .context }}
{{- $root := .root }}
{{- $path := .path }}
{{- $rootPath := default $path .rootPath }}

{{- if $context }}
  {{- $contextConfig = include "get-value-from-path" (dict "context" $context "path" $path) | fromYaml }}
{{- end }}

{{- if and $root $root.Values }}
  {{- $rootConfig = include "get-value-from-path" (dict "context" $root.Values "path" $rootPath) | fromYaml }}
{{- end }}

{{- if and $contextConfig (not (empty $contextConfig)) }}
  {{- toYaml $contextConfig }}
{{- else if and $rootConfig (not (empty $rootConfig)) }}
  {{- toYaml $rootConfig }}
{{- end }}
{{- end }}

{{/*
Set field and value with improved validation and fallback
Parameters:
  - field: Field name to set (required)
  - context: Context object (required)
  - root: Root context object for fallback (required)
  - path: Path to value (optional, defaults to "config.<field>")
Returns: Field and value if value exists and is not empty

Example Usage:
  # With explicit path:
  {{- include "set-field-and-value" (dict "field" "labels" "path" "config.labels" "context" $publisher "root" .) | nindent 6 }}

  # Without path (automatically uses "config.labels"):
  {{- include "set-field-and-value" (dict "field" "labels" "context" $publisher "root" .) | nindent 6 }}
*/}}
{{- define "set-field-and-value" -}}
{{- $path := default (printf "config.%s" .field) .path }}
{{- $rootPath := default $path .rootPath }}
{{- $value := include "get-context-value" (dict "context" .context "root" .root "path" $path "rootPath" $rootPath) | fromYaml }}
{{- if and $value (not (empty $value)) (not (eq (kindOf $value) "invalid")) }}
{{ .field }}:
  {{- toYaml $value | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Set value only with improved validation
This template retrieves a value from a nested YAML structure using a dot-path notation.

Parameters:
  - path: Path to value (required)
  - context: Context object (required)
  - root: Root context object for fallback (optional)

Returns: The YAML value if it exists and is valid, otherwise returns nothing

Example Usage:
  # With root fallback:
  {{- include "set-value" (dict "context" $publisher "root" . "path" "config.labels") | nindent 4 }}

  # Without root fallback:
  {{- include "set-value" (dict "context" $publisher "path" "config.labels") | nindent 4 }}

Input Example:
  publisher:
    config:
      labels:
        app: fuel-streams
        tier: backend

Result:
  app: fuel-streams
  tier: backend
*/}}
{{- define "set-value" -}}
{{- $path := default (printf "config.%s" .field) .path }}
{{- $rootPath := default $path .rootPath }}
{{- include "get-context-value" (dict "context" .context "root" .root "path" $path "rootPath" $rootPath) }}
{{- end }}
