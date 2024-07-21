use std::str::FromStr;

use alloy_primitives::{Address, Bytes, TxKind, U256};
use alloy_sol_types::SolCall;
use candid::Principal;
use ic_exports::ic_cdk::{
    self,
    api::{
        call::CallResult,
        management_canister::ecdsa::{EcdsaCurve, EcdsaKeyId},
    },
    print,
};
use serde_json::json;

use crate::{
    evm_rpc::{
        MultiSendRawTransactionResult, RequestResult, RpcApi, RpcService, RpcServices, Service,
    }, gas::{estimate_transaction_fees, FeeEstimates}, signer::{sign_eip1559_transaction, SignRequest}, types::{DerivationPath, EthCallResponse, RouterError}
};

pub fn rpc_provider(rpc_url: &str) -> RpcService {
    RpcService::Custom({
        RpcApi {
            url: rpc_url.to_string(),
            headers: None,
        }
    })
}

pub fn decode_request(
    canister_response: CallResult<(RequestResult,)>,
) -> Result<Vec<u8>, RouterError> {
    let decoded_response = decode_response(canister_response)?;
    match decoded_response {
        RequestResult::Ok(val) => {
            let response: EthCallResponse = serde_json::from_str(&val).unwrap();
            let cleaned_hex = &response.result[2..]; // Remove the "0x" prefix

            let padded_hex = if cleaned_hex.len() % 2 != 0 {
                format!("0{}", cleaned_hex) // Add a leading zero if necessary
            } else {
                String::from(cleaned_hex)
            };

            hex::decode(padded_hex).map_err(|error| RouterError::Unknown(error.to_string()))
        }
        RequestResult::Err(e) => Err(RouterError::RpcResponseError(e)),
    }
}

pub fn decode_response(
    canister_response: CallResult<(RequestResult,)>,
) -> Result<RequestResult, RouterError> {
    match canister_response {
        Ok((rpc_response,)) => Ok(rpc_response),
        Err(e) => Err(RouterError::Unknown(e.1)),
    }
}

pub async fn send_raw_transaction(
    to: String,
    data: Vec<u8>,
    value: U256,
    nonce: u64,
    derivation_path: DerivationPath,
    rpc_canister: &Service,
    rpc_url: &str,
    cycles: u128,
) -> Result<MultiSendRawTransactionResult, RouterError> {
    let input = Bytes::from(data);
    let rpc = RpcServices::Custom {
        chainId: 1,
        services: vec![RpcApi {
            url: rpc_url.to_string(),
            headers: None,
        }],
    };

    let FeeEstimates {
        max_fee_per_gas,
        max_priority_fee_per_gas,
    } = estimate_transaction_fees(9, rpc.clone(), rpc_canister).await?;

    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: String::from("key_1"),
    };

    let request = SignRequest {
        chain_id: 1,
        from: None,
        to: TxKind::Call(
            Address::from_str(&to)
                .map_err(|err| RouterError::DecodingError(format!("{:#?}", err)))?,
        ),
        max_fee_per_gas: max_fee_per_gas.to::<u128>(),
        max_priority_fee_per_gas: max_priority_fee_per_gas.to::<u128>(),
        value,
        nonce,
        data: input,
    };

    let signed_transaction = sign_eip1559_transaction(request, key_id, derivation_path).await;

    match rpc_canister
        .eth_send_raw_transaction(rpc, None, signed_transaction, cycles)
        .await
    {
        Ok((response,)) => Ok(response),
        Err(e) => Err(RouterError::Unknown(e.1)),
    }
}


// pub fn handle_rpc_response<T, F: SolCall<Return = T>>(
//     rpc_response: RequestResult,
// ) -> Result<T, ManagerError> {
//     match rpc_response {
//         RequestResult::Ok(hex_data) => {
//             let decoded_hex = hex::decode(hex_data)
//                 .map_err(|err| ManagerError::DecodingError(err.to_string()))?;
//             F::abi_decode_returns(&decoded_hex, false)
//                 .map_err(|err| ManagerError::DecodingError(err.to_string()))
//         }
//         RequestResult::Err(e) => Err(ManagerError::RpcResponseError(e)),
//     }
// }

// pub fn eth_call_args(to: String, data: Vec<u8>) -> String {
//     json!({
//         "id": 1,
//         "jsonrpc": "2.0",
//         "params": [ {
//             "to": to,
//             "data": format!("0x{}", hex::encode(data))
//         }
//         ],
//         "method": "eth_call"
//     })
//     .to_string()
// }

// pub async fn send_raw_transaction(
//     to: String,
//     data: String,
//     value: U256,
//     nonce: u64,
//     derivation_path: DerivationPath,
//     rpc_canister: &Service,
//     rpc_url: &str,
//     cycles: u128,
// ) -> Result<MultiSendRawTransactionResult, ManagerError> {
//     let rpc = RpcServices::Custom {
//         chainId: 1,
//         services: vec![RpcApi {
//             url: rpc_url.to_string(),
//             headers: None,
//         }],
//     };

//     let key_id = EcdsaKeyId {
//         curve: EcdsaCurve::Secp256k1,
//         name: String::from("key_1"),
//     };

//     let request = SignRequest {
//         chain_id: 1,
//         from: None,
//         to: TxKind::Call(Address::from_str(&to).unwrap()),
//         max_fee_per_gas: todo!(),
//         max_priority_fee_per_gas: todo!(),
//         value,
//         nonce,
//         data: Bytes::from_str(&data).unwrap(),
//     };

//     let signed_transaction = sign_eip1559_transaction(request, key_id, derivation_path).await;

//     match rpc_canister
//         .eth_send_raw_transaction(rpc, None, signed_transaction, cycles)
//         .await
//     {
//         Ok((response,)) => Ok(response),
//         Err(e) => Err(ManagerError::Custom(e.1)),
//     }
// }
