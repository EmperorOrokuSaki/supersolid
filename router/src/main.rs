mod canister;
mod evm_rpc;
mod signer;
mod state;
mod timers;
mod types;
mod utils;
mod gas;

use crate::canister::Supersolid;

fn main() {
    let canister_e_idl = Supersolid::idl();
    let idl = candid::pretty::candid::compile(&canister_e_idl.env.env, &Some(canister_e_idl.actor));

    println!("{}", idl);
}
