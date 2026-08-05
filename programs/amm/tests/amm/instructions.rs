use ::amm as amm_program;
use amm_program::{accounts, instruction};

use anchor_lang::{system_program, InstructionData, ToAccountMetas};
use solana_message::Instruction;

use crate::{
    amm::pdas::{find_protocol_config_pda, find_protocol_treasury_pda},
    common::context::TestContext,
};

pub fn initialize_protocol_ix(ctx: &TestContext) -> Instruction {
    let program_id = &ctx.program_id;

    let (protocol_config, _) = find_protocol_config_pda(program_id);
    let (protocol_treasury, _) = find_protocol_treasury_pda(program_id);

    let accounts = accounts::InitializeProtocol {
        admin: ctx.admin.pubkey(),
        protocol_config: protocol_config,
        protocol_treasury: protocol_treasury,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    let instruction = Instruction {
        program_id: *program_id,
        accounts,
        data: instruction::InitializeProtocol {
            swap_fee_bps: 30,
            treasury_fee_bps: 5,
        }
        .data(), // data not available in suggesstions. InstructionData import solved. why?
    };

    instruction
}
