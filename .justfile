arch := 'amd64'
profile := 'dev'
log := 'info'

export JUST_LOG := log

# Build a subproject: currently, only `zerOS` is available
build project='zerOS':
    @just --set arch {{ arch }} --set profile {{ profile }} _build-{{ project }}

_build-zerOS:
    @echo building zerOS for {{ arch }}
