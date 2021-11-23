use dia_oracle_runtime_api::CoinInfo;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

pub use dia_oracle_runtime_api::DiaOracleApi as DiaOracleRuntimeApi;

use std::sync::Arc;

#[rpc]
pub trait DiaOracleApi<BlockHash> {
	#[rpc(name = "dia_getCoinInfo")]
	fn get_coin_info(&self, name: Bytes, at: Option<BlockHash>) -> Result<CoinInfo>;
	#[rpc(name = "dia_getValue")]
	fn get_value(&self, name: Bytes, at: Option<BlockHash>) -> Result<u64>;
}

/// A struct that implements the [`DiaOracleApi`].
pub struct DiaOracleRpc<C, P> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

impl<C, P> DiaOracleRpc<C, P> {
	/// Create new `TransactionPayment` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

/// Error type of this RPC api.
pub enum Error {
	/// The transaction was not decodable.
	DecodeError,
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
			Error::DecodeError => 2,
		}
	}
}

impl<C, Block> DiaOracleApi<<Block as BlockT>::Hash> for DiaOracleRpc<C, Block>
where
	Block: BlockT,
	C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DiaOracleRuntimeApi<Block>,
{
	fn get_coin_info(&self, name: Bytes, at: Option<<Block as BlockT>::Hash>) -> Result<CoinInfo> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let r = api
			.get_coin_info(&at, name.to_vec())
			.map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query get_coin_info.".into(),
				data: Some(format!("{:?}", e).into()),
			})?
			.map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query get_coin_info.".into(),
				data: Some(format!("{:?}", e).into()),
			})?;

		Ok(r)
	}

	fn get_value(&self, name: Bytes, at: Option<<Block as BlockT>::Hash>) -> Result<u64> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		let r = api
			.get_value(&at, name.to_vec())
			.map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query get_value.".into(),
				data: Some(format!("{:?}", e).into()),
			})?
			.map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to query get_value.".into(),
				data: Some(format!("{:?}", e).into()),
			})?;
		Ok(r)
	}
}
