#!/usr/bin/env bash
set -eo pipefail

# Automatically generated by moon. DO NOT MODIFY!
# https://moonrepo.dev/docs/guides/vcs-hooks

moon sync projects
moon run :lint
moon run controller:test