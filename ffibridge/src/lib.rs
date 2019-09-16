use failure::prelude::*;

use std::thread;
use std::slice;
use protobuf::Message;
use serde_json::json;

use proto_conv::{IntoProto};
use types::{
    account_state_blob::AccountStateBlob,
    account_config::get_account_resource_or_default,
    account_address::{AccountAddress, ADDRESS_LENGTH}
};

fn bytes_from_c<'a>(buff: *const u8, len: usize) -> &'a[u8] {
    return unsafe { slice::from_raw_parts(buff, len as usize) };
}

fn bytes_from_c_mut<'a>(buff: *mut u8, len: usize) -> &'a mut [u8] {
    return unsafe { slice::from_raw_parts_mut(buff, len as usize) };
}

fn addr_from_bytes(data: &[u8]) -> Result<AccountAddress> {
    if data.len() == ADDRESS_LENGTH {
        return Err(format_err!("Invalid data length"));
    }
    let mut addr_array: [u8 ;ADDRESS_LENGTH] = Default::default();
    addr_array.copy_from_slice(&data[0..ADDRESS_LENGTH]);
    Ok(AccountAddress::new(addr_array))
}

fn pass_data(data: &[u8], out_data: *mut u8, out_size: *mut usize) -> Result<()> {
    let data_len = data.len();
    if out_data.is_null() {
        return Err(format_err!("Null out_data"));
    }
    if out_size.is_null() {
        return Err(format_err!("Null out_size"));
    }
    let buff_len = unsafe { *out_size };
    unsafe { *out_size = data_len };
    if data_len > buff_len {
        return Err(format_err!("No enough space in out_data (required: {}, available: {})", data_len, buff_len))
    }
    let view = bytes_from_c_mut(out_data, data_len);
    view.copy_from_slice(data);
    Ok(())
}

fn pass_string(str: &String, out_data: *mut u8, out_size: *mut usize) -> Result<()> {
    pass_data(str.as_bytes(), out_data, out_size)
}

fn pass_json(value: &serde_json::Value, out_data: *mut u8, out_size: *mut usize) -> Result<()> {
    let objstr = value.to_string();
    pass_string(&objstr, out_data, out_size)
}

// fn json_arr_from_bytes(data: &[u8]) -> Vec<serde_json::Number> {
//     data.into_iter().map(|x| { serde_json::Number::from(x.clone() as i32) }).collect()
// }

fn hex_from_bytes(data: &[u8]) -> String {
    hex::encode(data)
}

#[no_mangle]
pub extern fn encode_transfer_program(receiver_address_buff: *const u8, receiver_address_len: usize, num_coins: u64, program_buff: *mut u8, program_size: *mut usize) -> bool {
    if receiver_address_buff.is_null() {
        return false;
    }
    let addr_slice = bytes_from_c(receiver_address_buff, receiver_address_len);
    let addr = match addr_from_bytes(addr_slice) {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    let program = vm_genesis::encode_transfer_program(&addr, num_coins);
    // proto_conv::IntoProto is needed to use into_proto
    let program_proto : types::proto::transaction::Program = program.into_proto();
    let data = match program_proto.write_to_bytes() {
        Ok(data) => data,
        Err(_) => return false,
    };

    match pass_data(&data, program_buff, program_size) {
        Ok(()) => return true,
        Err(_) => return false,
    }
}

#[no_mangle]
pub extern fn get_allowed_scripts(out_json_buff: *mut u8, out_json_size: *mut usize) -> bool {
    match pass_json(&json!({
        "peer_to_peer_transaction": hex_from_bytes(&vm_genesis::PEER_TO_PEER_TXN),
        "create_account_transaction": hex_from_bytes(&vm_genesis::CREATE_ACCOUNT_TXN),
        "mint_transaction": hex_from_bytes(&vm_genesis::ROTATE_AUTHENTICATION_KEY_TXN),
        "rotate_authentication_key_transaction": hex_from_bytes(&vm_genesis::MINT_TXN),
    }), out_json_buff, out_json_size) {
        Ok(()) => true,
        Err(err) => { println!("{}", err.to_string()); false }
    }
}

#[no_mangle]
pub extern fn decode_account_state_blob(data: *const u8, size: usize, out_json_buff: *mut u8, out_json_size: *mut usize) -> bool {
    let blob_data = bytes_from_c(data, size);
    let blob = AccountStateBlob::from(blob_data.to_vec());

    let r = match get_account_resource_or_default(&Some(blob)) {
        Ok(r) => r,
        Err(_) => return false,
    };

    match pass_json(&json!({
        "balance": r.balance(),
        "sequence_number": r.sequence_number(),
        "authentication_key": hex_from_bytes(r.authentication_key().as_bytes()),
        "sent_events": {
            "key": hex_from_bytes(r.sent_events().key().as_bytes()),
            "count": r.sent_events().count(),
        },
        "received_events": {
            "key": hex_from_bytes(r.received_events().key().as_bytes()),
            "count": r.received_events().count(),
        },
        "delegated_withdrawal_capability": r.delegated_withdrawal_capability(),
    }), out_json_buff, out_json_size) {
        Ok(()) => true,
        Err(err) => { println!("{}", err.to_string()); false }
    }
}
