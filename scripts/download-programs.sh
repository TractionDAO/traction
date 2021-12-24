#!/usr/bin/env sh

cd $(dirname $0)/..

mkdir -p artifacts/programs/

# crate
curl -L https://github.com/CrateProtocol/crate/releases/download/v0.4.0/crate_token.so > \
    artifacts/programs/crate_token.so
