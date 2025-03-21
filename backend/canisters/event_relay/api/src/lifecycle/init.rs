use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use types::{BuildVersion, CanisterId};

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct Args {
    pub push_events_whitelist: Vec<Principal>,
    pub event_store_canister_id: CanisterId,
    pub cycles_dispenser_canister_id: CanisterId,
    pub registry_canister_id: CanisterId,
    pub chat_ledger_canister_id: CanisterId,
    pub chat_governance_canister_id: CanisterId,
    pub wasm_version: BuildVersion,
    pub test_mode: bool,
}
