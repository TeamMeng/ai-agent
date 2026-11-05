#!/usr/bin/env bash
# Runs a cargo based check, unless this is the template repository itself.
#
# The template keeps `{{project-name}}` / `{{authors}}` placeholders in
# Cargo.toml, which cargo cannot parse. Generated projects get real values, and
# CI normalizes the placeholders before running these checks (see
# .github/workflows/build.yml), so skipping here never hides a broken build.
set -euo pipefail

if grep -q '{{project-name}}' Cargo.toml 2>/dev/null; then
  echo "template placeholders in Cargo.toml: skipping '$*' (CI normalizes them)" >&2
  exit 0
fi

exec "$@"
