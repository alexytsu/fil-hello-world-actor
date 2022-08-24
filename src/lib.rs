mod blockstore;

use crate::blockstore::Blockstore;
use fil_fungible_token::token::state::TokenState;
use fvm_ipld_encoding::{to_vec, RawBytes, DAG_CBOR};
use fvm_sdk as sdk;
use fvm_sdk::NO_DATA_BLOCK_ID;
use fvm_shared::ActorID;
use sdk::sself;

/// A macro to abort concisely.
/// This should be part of the SDK as it's very handy.
macro_rules! abort {
    ($code:ident, $msg:literal $(, $ex:expr)*) => {
        fvm_sdk::vm::abort(
            fvm_shared::error::ExitCode::$code.value(),
            Some(format!($msg, $($ex,)*).as_str()),
        )
    };
}

/// The actor's WASM entrypoint. It takes the ID of the parameters block,
/// and returns the ID of the return value block, or NO_DATA_BLOCK_ID if no
/// return value.
///
/// Should probably have macros similar to the ones on fvm.filecoin.io snippets.
/// Put all methods inside an impl struct and annotate it with a derive macro
/// that handles state serde and dispatch.
#[no_mangle]
pub fn invoke(_: u32) -> u32 {
    // Conduct method dispatch. Handle input parameters and return data.
    let ret: Option<RawBytes> = match sdk::message::method_number() {
        1 => constructor(),
        2 => say_hello(),
        _ => abort!(USR_UNHANDLED_MESSAGE, "unrecognized method"),
    };

    // Insert the return data block if necessary, and return the correct
    // block ID.
    match ret {
        None => NO_DATA_BLOCK_ID,
        Some(v) => match sdk::ipld::put_block(DAG_CBOR, v.bytes()) {
            Ok(id) => id,
            Err(err) => abort!(USR_SERIALIZATION, "failed to store return value: {}", err),
        },
    }
}

/// The constructor populates the initial state.
///
/// Method num 1. This is part of the Filecoin calling convention.
/// InitActor#Exec will call the constructor on method_num = 1.
pub fn constructor() -> Option<RawBytes> {
    // This constant should be part of the SDK.
    const INIT_ACTOR_ADDR: ActorID = 1;

    // Should add SDK sugar to perform ACL checks more succinctly.
    // i.e. the equivalent of the validate_* builtin-actors runtime methods.
    // https://github.com/filecoin-project/builtin-actors/blob/master/actors/runtime/src/runtime/fvm.rs#L110-L146
    if sdk::message::caller() != INIT_ACTOR_ADDR {
        abort!(USR_FORBIDDEN, "constructor invoked by non-init actor");
    }

    let blockstore = Blockstore {};
    let token_state = TokenState::new(&blockstore).unwrap();
    let cid = token_state.save(&blockstore).unwrap();
    sself::set_root(&cid).unwrap();

    None
}

/// Method num 2.
pub fn say_hello() -> Option<RawBytes> {
    let ret = to_vec(format!("Hello world!").as_str()).unwrap();
    Some(RawBytes::new(ret))
}
