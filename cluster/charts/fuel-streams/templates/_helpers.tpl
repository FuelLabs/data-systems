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
Returns: Map - A set of key-value pairs representing Kubernetes labels
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
Core identifying labels used for object selection and service discovery.
These labels should be used consistently across all related resources.
Returns: Map - A set of key-value pairs for Kubernetes selector labels
*/}}
{{- define "fuel-streams.selectorLabels" -}}
app.kubernetes.io/name: {{ include "fuel-streams.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
Logic:
- If service account creation is enabled, use the fullname template (unless overridden)
- If service account creation is disabled, use the specified name or "default"
Returns: String - The name of the service account to use
*/}}
{{- define "fuel-streams.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "fuel-streams.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Merges configuration values between global defaults and service-specific overrides
Parameters:
  - service: Service name (e.g., "publisher")
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
Helper function to traverse and retrieve values from nested paths
Parameters:
  - context: The context object to traverse
  - path: Dot-notation string path (e.g., "config.resources.limits")
Example:
  {{- include "get-value-from-path" (dict "context" .context "path" .path) }}
Returns: The value at the specified path, or empty if not found
*/}}
{{- define "get-value-from-path" -}}
{{- $current := .context -}}
{{- range $part := splitList "." .path -}}
{{- if $current -}}
{{- $current = index $current $part -}}
{{- end -}}
{{- end -}}
{{- if $current -}}
{{- toYaml $current -}}
{{- end -}}
{{- end -}}

{{/*
Renders a YAML field with its key name, supporting nested path lookups
Parameters:
  - field: The field name to render (e.g., "nodeSelector", "tolerations")
  - path: Dot-notation path to the field's value in Values.yaml
  - context: The context to use (e.g., $publisher)
Example:
  {{- include "set-field-and-value" (dict "field" "nodeSelector" "path" "config.nodeSelector" "context" $publisher) }}
Returns:
  nodeSelector:
    key: value
*/}}
{{- define "set-field-and-value" -}}
{{- $value := include "get-value-from-path" . | fromYaml -}}
{{- if and $value (not (empty $value)) -}}
{{ .field -}}:
{{- toYaml $value | nindent 2 -}}
{{- end -}}
{{- end -}}

{{/*
Renders only the YAML value without the field name, supporting nested path lookups
Parameters:
  - field: Field name (used for context only)
  - path: Dot-notation path to the value in Values.yaml
  - context: The context to use (e.g., $publisher)
Example:
  {{- include "set-value" (dict "context" .context "path" .path) }}
Returns:
  key: value
*/}}
{{- define "set-value" -}}
{{- $value := include "get-value-from-path" . | fromYaml -}}
{{- if and $value (not (empty $value)) -}}
{{- toYaml $value | nindent 0 -}}
{{- end -}}
{{- end -}}

