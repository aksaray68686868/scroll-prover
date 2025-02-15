use crate::utils::{c_char_to_str, c_char_to_vec, vec_to_c_char, OUTPUT_DIR};
use libc::c_char;
use prover::{
    utils::init_env_and_log,
    zkevm::{Prover, Verifier},
    Proof,
};
use std::{cell::OnceCell, fs::File, io::Read};
use types::eth::BlockTrace;

static mut PROVER: OnceCell<Prover> = OnceCell::new();
static mut VERIFIER: OnceCell<Verifier> = OnceCell::new();

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn init_zkevm_prover(params_dir: *const c_char) {
    init_env_and_log("ffi_zkevm_prove");

    let params_dir = c_char_to_str(params_dir);
    let prover = Prover::from_params_dir(params_dir);

    PROVER.set(prover).unwrap();
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn init_zkevm_verifier(params_dir: *const c_char, vk_path: *const c_char) {
    init_env_and_log("ffi_zkevm_verify");

    let vk_path = c_char_to_str(vk_path);
    let mut f = File::open(vk_path).unwrap();
    let mut raw_vk = vec![];
    f.read_to_end(&mut raw_vk).unwrap();

    let params_dir = c_char_to_str(params_dir);
    let verifier = Verifier::from_params_dir(params_dir, &raw_vk);

    VERIFIER.set(verifier).unwrap();
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn gen_chunk_proof(block_traces: *const c_char) -> *const c_char {
    let block_traces = c_char_to_vec(block_traces);
    let block_traces = serde_json::from_slice::<Vec<BlockTrace>>(&block_traces).unwrap();

    let proof = PROVER
        .get_mut()
        .unwrap()
        .gen_chunk_proof(block_traces, OUTPUT_DIR.as_deref())
        .unwrap();

    let proof_bytes = serde_json::to_vec(&proof).unwrap();
    vec_to_c_char(proof_bytes)
}

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn verify_chunk_proof(proof: *const c_char) -> c_char {
    let proof = c_char_to_vec(proof);
    let proof = serde_json::from_slice::<Proof>(proof.as_slice()).unwrap();

    let verified = VERIFIER.get().unwrap().verify_chunk_proof(proof);
    verified as c_char
}
