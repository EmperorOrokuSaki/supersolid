#!/bin/bash

# Variables
API_URL="http://localhost:8787" # Replace with your actual worker URL
ALCHEMY_API_KEY="3Yon4ZMsceTLW54DCG3oLTFpzgbA7c2C" # Replace with your actual Alchemy API key
NETWORK="eth-mainnet" # Change to the desired network path

# JSON-RPC Request Payload
read -r -d '' JSON_RPC_REQUEST <<'EOF'
{
  "id": 1,
  "jsonrpc": "2.0",
  "method": "alchemy_getAssetTransfers",
  "params": [
    {
      "fromBlock": "0x0",
      "toBlock": "latest",
      "toAddress": "0x5c43B1eD97e52d009611D89b74fA829FE4ac56b1",
      "withMetadata": false,
      "excludeZeroValue": true,
      "maxCount": "0x3e8"
    }
  ]
}
EOF

# Make the POST request
response=$(curl --request POST \
     --url "$API_URL/$NETWORK" \
     --header "accept: application/json" \
     --header "content-type: application/json" \
     --header "Alchemy-Api-Key: $ALCHEMY_API_KEY" \
     --data "$JSON_RPC_REQUEST")

# Print the response
echo "Response from Worker:"
echo "$response"
