{{/*
Expand the name of the chart.
*/}}
{{- define "websurfx.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "websurfx.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "websurfx.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "websurfx.labels" -}}
helm.sh/chart: {{ include "websurfx.chart" . }}
{{ include "websurfx.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "websurfx.selectorLabels" -}}
app.kubernetes.io/name: {{ include "websurfx.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "websurfx.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "websurfx.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{- define "get.colorscheme" -}}
{{- $valid := list
    "catppuccin-mocha"
    "dark-chocolate"
    "dracula"
    "gruvbox-dark"
    "monokai"
    "nord"
    "oceanic-next"
    "one-dark"
    "solarized-dark"
    "solarized-light"
    "tokyo-night"
    "tomorrow-night"
-}}
{{- $default := "catppuccin-mocha" -}}
{{- $val := . | default $default -}}
{{- if not (has $val $valid) -}}
  {{- fail (printf "Invalid colorscheme: %s. Must be one of: %s" $val (join ", " $valid)) -}}
{{- end -}}
{{- ($val | quote) -}}
{{- end -}}

{{- define "get.theme" -}}
{{- $valid := list
    "simple"
-}}
{{- $default := "simple" -}}
{{- $val := . | default $default -}}
{{- if not (has $val $valid) -}}
  {{- fail (printf "Invalid theme: %s. Must be one of: %s" $val (join ", " $valid)) -}}
{{- end -}}
{{- ($val | quote) -}}
{{- end -}}

{{- define "get.animation" -}}
{{- $valid := list
    "nil"
	"simple-frosted-glow"
-}}
{{- $default := "simple-frosted-glow" -}}
{{- $val := . | default $default -}}
{{- if not (has $val $valid) -}}
  {{- fail (printf "Invalid animation: %s. Must be one of: %s" $val (join ", " $valid)) -}}
{{- end -}}
{{- if eq $val "nil" }}
{{- $val -}}
{{- else }}
{{- ($val | quote) -}}
{{- end }}
{{- end -}}

{{- define "get.bool" -}}
{{- $val := . -}}
{{- if not $val -}}
{{- "false" -}}
{{- else -}}
{{- "true" -}}
{{- end -}}
{{- end -}}

{{- define "websurfx.redis.internal.enabled" -}}
{{- if and .Values.websurfx.redis.enabled (not .Values.websurfx.redis.externalRedisUrl) -}}
true
{{- else -}}
false
{{- end -}}
{{- end }}

{{- define "websurfx.redis.port" -}}
{{- if and .Values.websurfx.redis.enabled (not .Values.websurfx.redis.externalRedisUrl) -}}
{{ .Values.redis.master.service.ports.redis }}
{{- else -}}
{{ .Values.websurfx.redis.port }}
{{- end }}
{{- end }}

{{- define "websurfx.redis.url" -}}
{{- if and .Values.websurfx.redis.enabled (not .Values.websurfx.redis.externalRedisUrl) -}}
redis://{{ .Release.Name }}-redis-master.{{ .Release.Namespace }}.svc.cluster.local:{{ .Values.redis.master.service.ports.redis }}
{{- end -}}
{{- if and .Values.websurfx.redis.enabled (.Values.websurfx.redis.externalRedisUrl) -}}
{{ .Values.websurfx.redis.externalRedisUrl }}:{{ .Values.websurfx.redis.port }}
{{- end -}}
{{- end -}}


{{- define "websurfx.proxy.url" -}}
{{- if not .Values.websurfx.server.proxy -}}
nil
{{- else -}}
{{ .Values.websurfx.server.proxy | quote }}
{{- end -}}
{{- end -}}
