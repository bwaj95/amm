use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ProtocolTreasury {
    pub bump: u8,
}
