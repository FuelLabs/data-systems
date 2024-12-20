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
Configure pod security context settings by merging global and service-specific values.
Parameters:
  - context: Root context for accessing global values
  - service: Service name for service-specific overrides
Returns: Security context configuration for pod-level settings
Example:
  {{- include "k8s.security-context" (dict "context" . "service" "publisher") }}
*/}}
{{- define "k8s.security-context" -}}
securityContext:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "securityContext" "path" "config.securityContext") | nindent 4 }}
{{- end }}

{{/*
Configure container security context settings by merging global and service-specific values.
Parameters:
  - context: Root context for accessing global values
  - service: Service name for service-specific overrides
Returns: Security context configuration for container-level settings
Example:
  {{- include "k8s.container-security-context" (dict "context" . "service" "publisher") }}
*/}}
{{- define "k8s.container-security-context" -}}
securityContext:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "containerSecurityContext" "path" "config.containerSecurityContext") | nindent 4 }}
{{- end }}

{{/*
Configure container probe settings by merging global and service-specific values.
Parameters:
  - context: Root context for accessing global values
  - service: Service name for service-specific overrides
Returns: Probe configurations for liveness, readiness, and startup
Example:
  {{- include "k8s.probes" (dict "context" . "service" "publisher") }}
*/}}
{{- define "k8s.probes" -}}
{{- if .context.Values.config.healthChecks }}
livenessProbe:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "livenessProbe" "path" "config.livenessProbe") | nindent 2 }}
readinessProbe:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "readinessProbe" "path" "config.readinessProbe") | nindent 2 }}
startupProbe:
  {{- include "merge" (dict "context" .context "service" .service "defaultKey" "startupProbe" "path" "config.startupProbe") | nindent 2 }}
{{- end }}
{{- end }}

{{/*
Configure nats accounts
*/}}
{{- define "nats-accounts" -}}
data:
  auth.conf: |
    accounts {
      SYS: {
        users: [
          {user: $NATS_SYS_USER, password: $NATS_SYS_PASS}
        ]
      }
      ADMIN: {
        jetstream: enabled
        users: [
          {user: $NATS_ADMIN_USER, password: $NATS_ADMIN_PASS}
        ]
      }
      PUBLIC: {
        jetstream: enabled
        users: [
          {
            user: $NATS_PUBLIC_USER
            password: $NATS_PUBLIC_PASS
            permissions: {
              subscribe: ">"
              publish: {
                deny: [
                  "*.by_id.>"
                  "*.blocks.>"
                  "*.transactions.>"
                  "*.inputs.>"
                  "*.outputs.>"
                  "*.receipts.>"
                  "*.logs.>"
                  "*.utxos.>"
                  "$JS.API.STREAM.CREATE.>"
                  "$JS.API.STREAM.UPDATE.>"
                  "$JS.API.STREAM.DELETE.>"
                  "$JS.API.STREAM.PURGE.>"
                  "$JS.API.STREAM.RESTORE.>"
                  "$JS.API.STREAM.MSG.DELETE.>"
                  "$JS.API.CONSUMER.DURABLE.CREATE.>"
                ]
              }
            }
          }
        ]
      }
    }
{{- end }}
