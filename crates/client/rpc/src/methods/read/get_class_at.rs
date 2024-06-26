use dc_db::storage_handler::primitives::contract_class::{ContractClassWrapper, StorageContractClassData};
use dc_db::storage_handler::StorageView;
use dp_convert::field_element::FromFieldElement;
use jsonrpsee::core::RpcResult;
use starknet_api::core::ContractAddress;
use starknet_core::types::{BlockId, ContractClass, FieldElement};

use crate::errors::StarknetRpcApiError;
use crate::utils::ResultExt;
use crate::{bail_internal_server_error, Starknet};

/// Get the Contract Class Definition at a Given Address in a Specific Block
///
/// ### Arguments
///
/// * `block_id` - The identifier of the block. This can be the hash of the block, its number
///   (height), or a specific block tag.
/// * `contract_address` - The address of the contract whose class definition will be returned.
///
/// ### Returns
///
/// * `contract_class` - The contract class definition. This may be either a standard contract class
///   or a deprecated contract class, depending on the contract's status and the blockchain's
///   version.
///
/// ### Errors
///
/// This method may return the following errors:
/// * `BLOCK_NOT_FOUND` - If the specified block does not exist in the blockchain.
/// * `CONTRACT_NOT_FOUND` - If the specified contract address does not exist.
pub fn get_class_at(
    starknet: &Starknet,
    block_id: BlockId,
    contract_address: FieldElement,
) -> RpcResult<ContractClass> {
    let block_number = starknet.get_block_n(block_id)?;
    let key = ContractAddress::from_field_element(contract_address);

    let class_hash = starknet
        .backend
        .contract_class_hash()
        .get_at(&key, block_number)
        .or_internal_server_error("Failed to retrieve contract class")?
        .ok_or(StarknetRpcApiError::ContractNotFound)?;

    // The class need to be stored
    let Some(contract_class_data) = starknet
        .backend
        .contract_class_data()
        .get(&class_hash)
        .or_internal_server_error("Failed to retrieve contract class from hash")?
    else {
        bail_internal_server_error!("Failed to retrieve contract class from hash")
    };

    // converting from stored Blockifier class to rpc class
    let StorageContractClassData { contract_class, abi, sierra_program_length, abi_length, block_number: _ } =
        contract_class_data;
    Ok(ContractClassWrapper { contract: contract_class, abi, sierra_program_length, abi_length }
        .try_into()
        .or_else_internal_server_error(|| {
            format!("Failed to convert contract class from hash '{class_hash}' to RPC contract class")
        })?)
}
