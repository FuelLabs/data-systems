{{/*
Configure default affinity settings for pod scheduling
*/}}
{{- define "k8s.pod-config.affinityy" -}}
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
Configure pod spec header including replicas and selector labels
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
  - name: Name of the service for selector labels
Returns: YAML configuration for pod spec header
*/}}
{{- define "k8s.pod-spec" -}}
{{- if not .context.autoscaling.enabled }}
replicas: {{ .context.config.replicaCount }}
{{- end }}
selector:
  matchLabels:
    {{- include "fuel-streams.selectorLabels" (dict "name" .name "context" .root) | nindent 4 }}
    app.kubernetes.io/component: {{ .component }}
{{- end }}

{{/*
Configure service account for pod
Parameters:
  - root: Root context object containing serviceAccount configuration
Returns: serviceAccountName if serviceAccount creation is enabled
*/}}
{{- define "k8s.pod-config.serviceAccount" -}}
{{- if .root.Values.serviceAccount.create }}
serviceAccountName: {{ include "fuel-streams.serviceAccountName" .root }}
{{- end }}
{{- end }}

{{/*
Configure image pull secrets for pod
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: imagePullSecrets configuration if specified
*/}}
{{- define "k8s.pod-config.imagePullSecrets" -}}
{{ include "set-field-and-value" (dict "context" .context "root" .root "field" "imagePullSecrets") -}}
{{- end }}

{{/*
Configure node selector for pod scheduling
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: nodeSelector configuration if specified
*/}}
{{- define "k8s.pod-config.nodeSelector" -}}
{{ include "set-field-and-value" (dict "context" .context "root" .root "field" "nodeSelector") -}}
{{- end }}

{{/*
Configure pod tolerations
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: tolerations configuration if specified
*/}}
{{- define "k8s.pod-config.tolerations" -}}
{{ include "set-field-and-value" (dict "context" .context "root" .root "field" "tolerations") -}}
{{- end }}

{{/*
Configure pod security context
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: securityContext configuration from config.podSecurityContext if specified
*/}}
{{- define "k8s.pod-config.securityContext" -}}
{{ include "set-field-and-value" (dict "context" .context "root" .root "field" "securityContext" "path" "config.podSecurityContext") -}}
{{- end }}

{{/*
Configure pod-level settings including security, scheduling and image pull configuration
Parameters:
  - root: Root context object for fallback values
  - context: Service-specific context object containing configuration
Returns: YAML configuration for pod-level settings
*/}}
{{- define "k8s.pod-config" -}}
{{ include "k8s.pod-config.serviceAccount" . }}
{{ include "k8s.pod-config.imagePullSecrets" . }}
{{ include "k8s.pod-config.nodeSelector" . }}
{{ include "k8s.pod-config.tolerations" . }}
{{ include "k8s.pod-config.securityContext" . }}
{{- end }}
