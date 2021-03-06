//! Program entrypoint

#![cfg_attr(feature = "strict", deny(warnings))]

use fund::{
    error::{FundError, FundErrorCode},
    instruction::FundInstruction,
};
use serum_common::pack::Pack;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

pub(crate) mod access_control;
mod close;
mod deposit;
mod initialize;
mod register_payback;
mod whitelist_add;
mod whitelist_delete;
mod withdraw;

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("process-instruction");

    let instruction: FundInstruction = FundInstruction::unpack(instruction_data)
        .map_err(|_| FundError::ErrorCode(FundErrorCode::WrongSerialization))?;

    let result = match instruction {
        FundInstruction::Initialize {
            owner,
            authority,
            max_balance,
            fund_type,
        } => initialize::handler(
            program_id,
            accounts,
            owner,
            authority,
            max_balance,
            fund_type,
        ),
        FundInstruction::Deposit { amount } => deposit::handler(program_id, accounts, amount),
        FundInstruction::Withdraw { amount } => withdraw::handler(program_id, accounts, amount),
        FundInstruction::Close => close::handler(program_id, accounts),
        FundInstruction::WhitelistAdd { entry } => {
            whitelist_add::handler(program_id, accounts, entry)
        }
        FundInstruction::WhitelistDelete { entry } => {
            whitelist_delete::handler(program_id, accounts, entry)
        }
        FundInstruction::RegisterPayback { amount } => {
            register_payback::handler(program_id, accounts, amount)
        }
    };

    result?;

    msg!("process-instruction success");

    Ok(())
}
