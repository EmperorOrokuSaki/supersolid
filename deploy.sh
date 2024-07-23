#!/bin/bash

dfx deploy --ic router

dfx canister start --ic router

sh poll_logs.sh