{{/*
Configure pod template metadata including annotations and labels
Parameters:
  - root: Root context object for fallback values 
  - context: Service-specific context object containing configuration
  - name: Name of the service for labels
  - component: Optional component name for labels
Returns: YAML configuration for pod template metadata
*/}}
{{- define "k8s.template-labels" -}}
labels:
  {{- $component := .component | default .name }}
  {{- include "fuel-streams.labels" (dict "name" .name "context" .root) | nindent 2 }}
  {{- include "set-value" (dict "context" .context "root" .root "path" "config.labels") | nindent 2 }}
  {{- if $component }}
  app.kubernetes.io/component: {{ $component }}
  {{- end }}
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
{{- include "set-field-and-value" (dict "context" .context "root" .root "field" "annotations" "path" "config.podAnnotations") }}
{{- end }}

{{/*
Configure pod template metadata including annotations and labels

Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
  - component: Optional component name for labels
  - name: Name of the service for labels
Returns: YAML configuration for pod template metadata including:
  - Annotations from config.podAnnotations
  - Labels from k8s.template-labels helper
*/}}
{{- define "k8s.template-metadata" -}}
metadata:
  {{- include "k8s.template-annotations" . | nindent 2 }}
  {{- include "k8s.template-labels" . | nindent 4 }}
{{- end }}
