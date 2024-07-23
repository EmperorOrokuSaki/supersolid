#!/bin/bash

dfx deploy --ic router

dfx canister start --ic router

## BASE, OPTIMISM, ARBITRUM, ETH MAINNET
# dfx canister call --ic router start "(principal \"7hfb6-caaaa-aaaar-qadga-cai\", vec { record {\"https://eth-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 1}; record {\"https://opt-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 10}; record {\"https://base-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 8453};record {\"https://arb-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 42161}})"

## ONLY BASE MAINNET
dfx canister call --ic router start "(vec { record {\"https://base-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 8453}; record {\"https://arb-mainnet.g.alchemy.com/v2/3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C\"; 42161}})" 
# dfx canister call --ic router add_request "(\"0x02f384dc0539FF16B820239004D25Dae1e3aDf96\", \"0x000000000000a4b102f384dc0539ff16b820239004d25dae1e3adf9600005af3107a4000\", principal \"zydig-qiaaa-aaaal-ajn6a-cai\")"

sh poll_logs.sh