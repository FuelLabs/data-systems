{{- define "fuel-streams.p2p_address" -}}
{{- if not .Values.publisher.reservedNodes }}
  {{- if eq .Values.publisher.network "mainnet" }}
    /dnsaddr/mainnet.fuel.network
  {{- else if eq .Values.publisher.network "testnet" }}
    /dnsaddr/testnet.fuel.network
  {{- else if eq .Values.publisher.network "devnet" }}
    /dnsaddr/devnet.fuel.network
  {{- end }}
{{- else }}
  {{- .Values.publisher.reservedNodes }}
{{- end }}
{{- end -}}
