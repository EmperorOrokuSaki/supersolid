#!/bin/bash

dfx deploy --ic router

dfx canister start --ic router

dfx canister call --ic router start "(principal \"7hfb6-caaaa-aaaar-qadga-cai\", vec { record {\"https://eth-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 1}; record {\"https://opt-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 10}; record {\"https://arb-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 42161}})"

sh poll_logs.sh