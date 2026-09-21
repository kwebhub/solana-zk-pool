use super::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke};

#[derive(Accounts)]
#[instruction(proof: Vec<u8>, nullifier_hash: [u8; 32], root: [u8; 32], to: Pubkey, amount: u64)]
pub struct InitWithdraw<'info> {
    #[account(mut, seeds = [POOL_SEED], bump)]
    pub pool: Account<'info, PoolState>,

    /// NullifierRecord PDA — created on first withdraw for this nullifier_hash.
    #[account(
        init,
        payer = payer,
        space = 8 + NullifierRecord::INIT_SPACE,
        seeds = [NullifierRecord::SEED, pool.key().as_ref(), nullifier_hash.as_ref()],
        bump,
    )]
    pub nullifier_record: Account<'info, NullifierRecord>,

    #[account(mut, seeds = [VAULT_SEED, pool.key().as_ref()], bump)]
    pub pool_vault: SystemAccount<'info>,

    /// CHECK: Validated against `to` parameter.
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,

    /// Payer for NullifierRecord creation. Must be signer.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Validated by constraint.
    #[account(constraint = verifier_program.key() == VERIFIER_PROGRAM_ID @ PtransError::InvalidVerifier)]
    pub verifier_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler_withdraw(
    ctx: Context<InitWithdraw>,
    proof: Vec<u8>,
    nullifier_hash: [u8; 32],
    root: [u8; 32],
    to: Pubkey,
    amount: u64,
    recipient_binding: [u8; 32],
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    require!(
        ctx.accounts.recipient.key() == to,
        PtransError::RecipientMismatch
    );
    require!(
        ctx.accounts.pool_vault.lamports() >= amount,
        PtransError::InsufficientVaultBalance
    );
    require!(pool.is_known_root(&root), PtransError::InvalidRoot);

    // Verify ZK proof
    let public_inputs =
        encode_public_inputs(&root, &nullifier_hash, &to, &recipient_binding, amount);

    let verify_instruction = Instruction {
        program_id: VERIFIER_PROGRAM_ID,
        accounts: vec![],
        data: [proof, public_inputs].concat(),
    };

    invoke(
        &verify_instruction,
        &[ctx.accounts.verifier_program.to_account_info()],
    )
    .map_err(|_| PtransError::InvalidProof)?;

    // Record nullifier (created via init — would fail if already exists)
    let record = &mut ctx.accounts.nullifier_record;
    record.pool = pool.key();
    record.nullifier_hash = nullifier_hash;
    record.recipient = to;
    record.amount = amount;
    record.timestamp = clock.unix_timestamp;

    // Transfer SOL from vault
    let vault_bump = ctx.bumps.pool_vault;
    let pool_key = pool.key();
    let vault_seeds = &[VAULT_SEED, pool_key.as_ref(), &[vault_bump]];
    let signer_seeds = &[&vault_seeds[..]];

    let cpi_context = CpiContext::new_with_signer(
        *ctx.accounts.system_program.key,
        system_program::Transfer {
            from: ctx.accounts.pool_vault.to_account_info(),
            to: ctx.accounts.recipient.to_account_info(),
        },
        signer_seeds,
    );
    system_program::transfer(cpi_context, amount)?;

    emit!(WithdrawEvent {
        nullifier_hash,
        recipient: ctx.accounts.recipient.key(),
        amount,
        timestamp: clock.unix_timestamp,
    });

    msg!("Withdrawal: {} lamports to {}", amount, to);
    Ok(())
}
