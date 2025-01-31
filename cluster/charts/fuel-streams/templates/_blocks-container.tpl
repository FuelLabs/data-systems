{{/*
Configure container resource requests and limits
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
*/}}
{{ define "k8s.container-config.resources" -}}
{{- include "set-field-and-value" (dict "context" .context "root" .root "field" "resources") -}}
{{- end }}

{{/*
Configure container security context settings
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
*/}}
{{- define "k8s.container-config.securityContext" -}}
{{- include "set-field-and-value" (dict "context" .context "root" .root "field" "securityContext" "path" "config.containerSecurityContext") -}}
{{- end }}

{{/*
Configure container health check probes
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
*/}}
{{- define "k8s.container-config.probes" -}}
{{- if .root.Values.config.healthChecks }}
{{- include "set-field-and-value" (dict "context" .context "root" .root "field" "startupProbe") -}}
{{- include "set-field-and-value" (dict "context" .context "root" .root "field" "livenessProbe") -}}
{{- include "set-field-and-value" (dict "context" .context "root" .root "field" "readinessProbe") -}}
{{- end }}
{{- end }}

{{- define "k8s.container-config.ports" -}}
ports:
{{- if .context.port }}
- name: http
  containerPort: {{ .context.port }}
  protocol: TCP
{{- end }}
{{- with .context.ports }}
{{ toYaml . | nindent 0 }}
{{- end }}
{{- end }}

{{/*
Configure container environment variables
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration with port and env map
*/}}
{{- define "k8s.container-config.env" -}}
env:
{{- if .context.port }}
- name: PORT
  value: {{ .context.port | quote }}
{{- end }}
{{- range $key, $value := .context.env }}
- name: {{ $key }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}

{{/*
Configure container environment from external sources
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration with envFrom
*/}}
{{- define "k8s.container-config.envFrom" -}}
envFrom:
- configMapRef:
    name: {{ include "fuel-streams.fullname" .root }}-config
    optional: true
- secretRef:
    name: {{ include "fuel-streams.fullname" .root }}-keys
    optional: true
{{- with .context.envFrom }}
{{ toYaml . | nindent 0 }}
{{- end }}
{{- end }}

{{/*
Configure container image settings
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing image configuration
*/}}
{{- define "k8s.container-config.image" -}}
image: "{{ .context.image.repository }}:{{ .context.image.tag | default .root.Chart.AppVersion }}"
imagePullPolicy: {{ .context.image.pullPolicy }}
{{- end }}

{{/*
Configure container-level settings including resource requests, security context, and probes
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
  - component: Optional component name for labels
  - name: Name of the service for labels
Returns: YAML configuration for container-level settings
*/}}
{{- define "k8s.container-config" -}}
{{ include "k8s.container-config.resources" . }}
{{ include "k8s.container-config.securityContext" . }}
{{ include "k8s.container-config.probes" . }}
{{ include "k8s.container-config.ports" . }}
{{ include "k8s.container-config.env" . }}
{{ include "k8s.container-config.envFrom" . }}
{{- end }}
