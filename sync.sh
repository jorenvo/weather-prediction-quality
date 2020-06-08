#!/usr/bin/env bash
set -euo pipefail

rsync -avP --exclude 'node_modules' --exclude 'target/debug/deps' --exclude 'target/debug/build' --exclude 'target/debug/incremental' --exclude 'target/rls' . jvo:/tmp/weather/
