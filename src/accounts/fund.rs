use serde::{Deserialize, Serialize};
use serum_common::pack::*;
use solana_client_gen::solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FundType {
  /// similar to a gofundme
  FundMe,
  // Raise,
}

/// Initialized program details.
/// Fund is a program account.
/// The Owner of the fund has the right to withdraw all or some of the funds
#[derive(Debug, Serialize, Deserialize)]
pub struct Fund {
  /// open defines if a fund is open for deposits
  pub open: bool,
  /// type of fund
  pub fund_type: FundType,
  /// fund Owner
  pub owner: Pubkey,
  /// Owner authority
  pub authority: Pubkey,
  /// max size of the fund
  pub max_balance: u32,
  /// balance of the
  pub balance: u32,
  /// Nonce of the program account
  pub nonce: u8,
  /// Mint
  pub mint: Pubkey,
  /// Address of the token vault controlled by the Safe.
  pub vault: Pubkey,
}

serum_common::packable!(Fund);