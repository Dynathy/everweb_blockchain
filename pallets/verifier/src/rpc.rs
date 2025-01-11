use jsonrpc_core::{Error as RpcError, Result};
use jsonrpc_derive::rpc;
use sp_runtime::traits::Block as BlockT;
use sp_core::{H256, crypto::AccountId32};
use std::sync::Arc;

#[rpc]
pub trait ValidatorRpc {
    /// Fetch the list of assigned submissions for a validator.
    #[rpc(name = "validator_getAssignedSubmissions")]
    fn get_assigned_submissions(&self, validator: AccountId32) -> Result<Vec<H256>>;

    /// Submit the validation result for a specific hash.
    #[rpc(name = "validator_submitValidation")]
    fn submit_validation(&self, validator: AccountId32, hash: H256, is_valid: bool) -> Result<bool>;
}

pub struct ValidatorRpcImpl<C> {
    client: Arc<C>,
}

impl<C> ValidatorRpcImpl<C> {
    pub fn new(client: Arc<C>) -> Self {
        Self { client }
    }
}

impl<C, Block> ValidatorRpc for ValidatorRpcImpl<C>
where
    C: ProvideRuntimeApi<Block>,
    C::Api: pallet_validator_runtime_api::ValidatorRuntimeApi<Block, AccountId32, H256>,
    Block: BlockT,
{
    fn get_assigned_submissions(&self, validator: AccountId32) -> Result<Vec<H256>> {
        let api = self.client.runtime_api();
        let at = self.client.info().best_hash;
        api.get_assigned_submissions(at, validator).map_err(|e| RpcError::from(e))
    }

    fn submit_validation(&self, validator: AccountId32, hash: H256, is_valid: bool) -> Result<bool> {
        let api = self.client.runtime_api();
        let at = self.client.info().best_hash;
        api.submit_validation(at, validator, hash, is_valid)
            .map(|_| true)
            .map_err(|e| RpcError::from(e))
    }
}
