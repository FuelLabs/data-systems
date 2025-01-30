{{/*
Configure resource annotations
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: Annotations from config.annotations
*/}}
{{- define "k8s.resource-metadata.annotations-raw" -}}
{{ include "set-value" (dict "root" .root "context" .context "path" "config.annotations") }}
{{- end }}

{{/*
Configure resource labels
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
  - path: Path to labels configuration ("config.labels")
Returns: Labels from config.labels
*/}}
{{- define "k8s.resource-metadata.labels-raw" -}}
{{ include "set-value" (dict "root" .root "context" .context "path" "config.labels") }}
{{- end }}

{{/*
Configure default resource metadata
Parameters:
  - root: Root context object containing Release and Chart info
  - name: Name to use for the resource
Returns: Basic resource metadata including:
  - name: Fully qualified name combining root name and resource name
  - namespace: Release namespace
  - app: Chart name
*/}}
{{- define "k8s.resource-metadata.default" -}}
{{ $fullname := include "fuel-streams.fullname" .root -}}
{{ $name := printf "%s-%s" $fullname .name -}}
name: {{ $name }}
{{- if .root.Values.namespace.enabled -}}
namespace: {{ default .root.Release.Namespace .root.Values.namespace.name }}
{{- else -}}
namespace: {{ .root.Release.Namespace }}
{{- end }}
app: {{ .root.Chart.Name }}
{{- end }}

{{/*
Configure resource annotations with proper indentation
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
  - name: Name of the service
  - component: Optional component name
Returns: Annotations block with annotations from config.annotations properly indented
*/}}
{{- define "k8s.resource-metadata.annotations" -}}
{{- $annotations := include "k8s.resource-metadata.annotations-raw" . | fromYaml }}
{{- if $annotations }}
annotations:
  {{- toYaml $annotations | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Configure resource labels
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
  - name: Name of the service for labels
  - component: Optional component name for labels
Returns: Labels including common labels, custom labels from config.labels, and component if specified
*/}}
{{- define "k8s.resource-metadata.labels" -}}
labels:
  {{- include "fuel-streams.labels" (dict "name" .name "context" .root) | nindent 2 }}
  {{- if .component }}
  app.kubernetes.io/component: {{ .component }}
  {{- end }}
  {{- $labels := include "k8s.resource-metadata.labels-raw" . | fromYaml }}
  {{- if $labels }}
  {{- toYaml $labels | nindent 2 }}
  {{- end }}
{{- end }}

{{/*
Configure resource metadata including name, namespace, labels and annotations
Parameters:
  - context: Root context object containing Release and Chart info
  - name: Name to use for the resource
  - suffix: Optional suffix to append to resource name
  - component: Optional component name for labels
Returns: Complete resource metadata configuration
*/}}
{{- define "k8s.resource-metadata" -}}
{{ include "k8s.resource-metadata.default" . }}
{{ include "k8s.resource-metadata.labels" . }}
{{ include "k8s.resource-metadata.annotations" . }}
{{- end }}
