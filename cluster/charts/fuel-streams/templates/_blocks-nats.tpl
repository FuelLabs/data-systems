{{/*
* NATS default accounts
*/}}
{{- define "nats-accounts" -}}
data:
  auth.conf: |
    accounts {
      SYS: {
        users: [
          {user: $NATS_SYSTEM_USER, password: $NATS_SYSTEM_PASS}
        ]
      }
      ADMIN: {
        jetstream: enabled
        users: [
          {user: $NATS_ADMIN_USER, password: $NATS_ADMIN_PASS}
        ]
      }
    }
{{- end }}