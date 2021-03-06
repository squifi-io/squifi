//! Program state processor

use crate::access_control;
use fund::{
    accounts::{
        fund::{Fund, FundType},
        vault::TokenVault,
    },
    error::{FundError, FundErrorCode},
};
use serum_common::pack::Pack;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    msg,
    program_option::COption,
    pubkey::Pubkey,
};
use std::convert::Into;

pub fn handler(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    owner: Pubkey,
    authority: Pubkey,
    max_balance: u64,
    fund_type: FundType,
) -> Result<(), FundError> {
    msg!("Initialize Fund");

    let acc_infos = &mut accounts.iter();
    let fund_acc_info = next_account_info(acc_infos)?;
    let vault_acc_info = next_account_info(acc_infos)?;
    let mint_acc_info = next_account_info(acc_infos)?;
    let rent_acc_info = next_account_info(acc_infos)?;

    // Optional accounts
    let whitelist_acc_info = acc_infos.next();
    let nft_token_acc_info = acc_infos.next();
    let nft_mint_acc_info = acc_infos.next();

    access_control(AccessControlRequest {
        program_id,
        fund_acc_info,
        mint_acc_info,
        vault_acc_info,
        rent_acc_info,
        nft_mint_acc_info,
        nonce: 0,
    })?;

    // 2. Creation
    msg!("create fund");
    Fund::unpack_mut(
        &mut fund_acc_info.try_borrow_mut_data()?,
        &mut |fund_acc: &mut Fund| {
            state_transition(StateTransitionRequest {
                fund_acc,
                owner,
                authority,
                mint: mint_acc_info.key,
                nft_mint_acc_info,
                nft_token_acc_info,
                vault: *vault_acc_info.key,
                whitelist_acc_info,
                fund_type,
                nonce: 0,
                max_balance,
            })
            .map_err(Into::into)
        },
    )?;

    Ok(())
}

fn access_control(req: AccessControlRequest) -> Result<(), FundError> {
    msg!("access-control: initialize");

    let AccessControlRequest {
        program_id,
        fund_acc_info,
        mint_acc_info,
        rent_acc_info,
        nft_mint_acc_info,
        vault_acc_info,
        nonce,
    } = req;

    let rent = access_control::rent(rent_acc_info)?;

    let fund = Fund::unpack(&fund_acc_info.try_borrow_data()?)?;
    {
        if fund_acc_info.owner != program_id {
            return Err(FundErrorCode::NotOwnedByProgram.into());
        }
        if !rent.is_exempt(fund_acc_info.lamports(), fund_acc_info.try_data_len()?) {
            return Err(FundErrorCode::NotRentExempt.into());
        }
        if fund.initialized {
            return Err(FundErrorCode::AlreadyInitialized.into());
        }
    }

    {
        let vault = access_control::token(vault_acc_info)?;
        let vault_authority = Pubkey::create_program_address(
            &TokenVault::signer_seeds(fund_acc_info.key, &nonce),
            program_id,
        )
        .map_err(|_| FundErrorCode::InvalidVaultNonce)?;
        if vault.owner != vault_authority {
            return Err(FundErrorCode::InvalidVault.into());
        }
    }

    if fund.fund_type.eq(&FundType::Raise { private: false })
        || fund.fund_type.eq(&FundType::Raise { private: true })
    {
        let nft_mint = access_control::mint(nft_mint_acc_info.unwrap())?;
        let fund_authority = Pubkey::create_program_address(
            &TokenVault::signer_seeds(&fund_acc_info.key, &fund.nonce),
            program_id,
        )
        .map_err(|_| FundErrorCode::InvalidVaultNonce)?;
        if nft_mint.mint_authority != COption::Some(fund_authority) {
            return Err(FundErrorCode::InvalidMintAuthority.into());
        }
    }

    // Mint (initialized but not yet on Safe).
    let _ = access_control::mint(mint_acc_info)?;

    msg!("access-control: success");

    Ok(())
}

fn state_transition(req: StateTransitionRequest) -> Result<(), FundError> {
    msg!("state-transition: initialize");

    let StateTransitionRequest {
        fund_acc,
        owner,
        authority,
        vault,
        mint,
        nft_mint_acc_info,
        nft_token_acc_info,
        fund_type,
        nonce,
        max_balance,
        whitelist_acc_info,
    } = req;

    fund_acc.initialized = true;
    fund_acc.open = true;
    fund_acc.owner = owner;
    fund_acc.authority = authority;
    fund_acc.vault = vault;
    fund_acc.mint = *mint;
    fund_acc.max_balance = max_balance;
    fund_acc.balance = 0;
    fund_acc.fund_type = fund_type;
    fund_acc.nonce = nonce;

    if fund_type.eq(&FundType::Raise { private: false })
        || fund_type.eq(&FundType::Raise { private: true })
    {
        fund_acc.nft_mint = *nft_mint_acc_info.unwrap().key;
        fund_acc.nft_account = *nft_token_acc_info.unwrap().key;
        fund_acc.round = 0u32;
    }
    if fund_type.eq(&FundType::Raise { private: true }) {
        fund_acc.whitelist = *whitelist_acc_info.unwrap().key;
    }

    msg!("state-transition: success");

    Ok(())
}

struct AccessControlRequest<'a, 'b> {
    program_id: &'a Pubkey,
    fund_acc_info: &'a AccountInfo<'b>,
    mint_acc_info: &'a AccountInfo<'b>,
    rent_acc_info: &'a AccountInfo<'b>,
    nft_mint_acc_info: Option<&'a AccountInfo<'b>>,
    vault_acc_info: &'a AccountInfo<'b>,
    nonce: u8,
}

struct StateTransitionRequest<'a, 'b> {
    fund_acc: &'a mut Fund,
    owner: Pubkey,
    mint: &'a Pubkey,
    whitelist_acc_info: Option<&'a AccountInfo<'b>>,
    nft_token_acc_info: Option<&'a AccountInfo<'b>>,
    nft_mint_acc_info: Option<&'a AccountInfo<'b>>,
    vault: Pubkey,
    authority: Pubkey,
    fund_type: FundType,
    nonce: u8,
    max_balance: u64,
}
