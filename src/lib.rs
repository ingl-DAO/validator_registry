pub mod add_program;
pub mod state;
pub mod utils;

use crate::add_program::{add_marketplace_program, add_permissionless_validator_program};
use crate::state::constants::MARKETPLACE_STORAGE_SEED;
use crate::state::{constants, NameStorage, Storage, MarketplaceStorage};
use crate::utils::ResultExt;
use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solana_program::{entrypoint, msg};
use utils::AccountInfoHelpers;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum InstructionEnum {
    InitConfig,
    AddValidatorProgram { name: String },
    RemovePrograms { program_count: u8 },
    AddMarketplaceProgram,
    Reset,
    Blank,
}
impl InstructionEnum {
    pub fn decode(input: &[u8]) -> Self {
        let instruction = InstructionEnum::try_from_slice(input).unwrap();
        instruction
    }
}

entrypoint!(process_instruction);

pub fn process_instruction(
    //TODO: use slicing for storage account storing and fetching.
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &constants::ID);
    let instruction = InstructionEnum::decode(input);
    match instruction {
        InstructionEnum::InitConfig => {
            // let account_info_iter = &mut accounts.iter();
            // let payer_account_info = next_account_info(account_info_iter)?;
            // let config_account = next_account_info(account_info_iter)?;
            // let (_config_key, config_bump) =
            //     config_account.assert_seed(program_id, &[b"config"])?;
            // let config_data = Config::default();
            // invoke_signed(
            //     &system_instruction::create_account(
            //         payer_account_info.key,
            //         config_account.key,
            //         Rent::get()?.minimum_balance(config_data.get_space()),
            //         config_data.get_space() as u64,
            //         program_id,
            //     ),
            //     &[payer_account_info.clone(), config_account.clone()],
            //     &[&[b"config", &[config_bump]]],
            // )?;
            // config_data.serialize(&mut &mut config_account.data.borrow_mut()[..])?;
        }
        InstructionEnum::AddValidatorProgram { name } => {
            add_permissionless_validator_program(program_id, accounts, name)?
        }
        InstructionEnum::AddMarketplaceProgram => add_marketplace_program(program_id, accounts)?,

        InstructionEnum::Reset => {
            let account_info_iter = &mut accounts.iter();
            let _payer_account_info = next_account_info(account_info_iter)?;
            let storage_account = next_account_info(account_info_iter)?;
            let name_storage_account = next_account_info(account_info_iter)?;
            let marketplace_storage_account = next_account_info(account_info_iter)?;

            let (_storage_key, _storage_bump) = storage_account
                .assert_seed(program_id, &[b"storage"])
                .error_log("Error @ storage_account assertion")?;

            let (_name_storage_key, _name_storage_bump) = name_storage_account
                .assert_seed(program_id, &[b"name_storage"])
                .error_log("Error @ name_storage_account assertion")?;

            let (_marketplace_storage_key, _marketplace_storage_bump) = marketplace_storage_account
                .assert_seed(program_id, &[MARKETPLACE_STORAGE_SEED])
                .error_log("Error @ marketplace_storage_account assertion")?;

            let name_storage_data = NameStorage::default();

            let storage_data = Storage::default();

            let marketplace_data = MarketplaceStorage::default();

            msg!(
                "Resetting storages, name_storage: {:?}, storage: {:?} marketplace_storage: {:?}",
                storage_account.data_len(),
                name_storage_account.data_len(),
                marketplace_storage_account.data_len(),
            );

            // config.validator_numeration = 0;

            storage_data
                .serialize(&mut &mut storage_account.data.borrow_mut()[..])
                .error_log("Error @ storage serialization")?;

            name_storage_data
                .serialize(&mut &mut name_storage_account.data.borrow_mut()[..])
                .error_log("Error @ name storage serialization")?;

            marketplace_data
                .serialize(&mut &mut marketplace_storage_account.data.borrow_mut()[..])
                .error_log("Error @ marketplace storage serialization")?;
            // config
            //     .serialize(&mut &mut config_account.data.borrow_mut()[..])
            //     .error_log("Error @ config account data serialization")?;
        }

        _ => Err(ProgramError::InvalidInstructionData)?,
    }
    Ok(())
}
