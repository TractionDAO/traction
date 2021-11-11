#!/usr/bin/env sh

cd $(dirname $0)/..

mkdir -p artifacts/programs/

# crate
solana program dump CRATwLpu6YZEeiVq9ajjxs61wPQ9f29s1UoQR9siJCRs \
    artifacts/programs/crate_token.so --url mainnet-beta
