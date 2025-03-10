use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer};

mod model;
mod utils;
use model::*;
use utils::*;

declare_id!("7D5MCa7qRv8wpbnycWVqDfgo8pZj5v2ghqvD2vy2jLiH");

#[program]
pub mod lending_pool {
    use super::*;

    // 初始化资金池
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        mint: Pubkey,
        decimals: u8,
        reserve_factor: u8,
        collateral_factor: u8,
        base_rate: u64,
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;

        // 设置初始参数
        pool.mint = mint;
        pool.decimals = decimals;
        pool.reserve_factor = reserve_factor;
        pool.collateral_factor = collateral_factor;
        pool.base_rate = base_rate;

        // 初始化全局状态
        pool.total_supply = 0;
        pool.total_borrowed = 0;
        pool.liquidity_index = 1_000_000_000; // 初始流动性指数
        pool.borrow_index = 1_000_000_000; // 初始借款指数
        pool.last_update_time = Clock::get()?.unix_timestamp;

        Ok(())
    }

    // 存款
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        // 1. 更新全局利息
        accrue_interest(&mut ctx.accounts.pool)?;

        // 2. 更新用户仓位
        let user_position = &mut ctx.accounts.user_position;
        user_position.deposited_amount = user_position
            .deposited_amount
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        // 3. 更新资金池
        let pool = &mut ctx.accounts.pool;
        pool.total_supply = pool
            .total_supply
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        // 4. 转移代币到资金池
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // 5. 记录事件
        emit!(LendingEvent {
            event_type: 0, // 0=存款
            amount,
            user: ctx.accounts.user.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // 借款
    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        // 1. 更新全局利息
        accrue_interest(&mut ctx.accounts.pool)?;

        // 2. 借款前检查
        check_before_borrow(&ctx, amount)?;

        // 3. 更新用户借款
        let user_position = &mut ctx.accounts.user_position;
        user_position.borrowed_amount = user_position
            .borrowed_amount
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        // 4. 更新资金池
        let pool = &mut ctx.accounts.pool;
        pool.total_borrowed = pool
            .total_borrowed
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        // 5. 将代币从资金池转移到用户账户
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
            ),
            amount,
        )?;

        // 6. 记录事件
        emit!(LendingEvent {
            event_type: 2, // 2=借款
            amount,
            user: ctx.accounts.user.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        // 1. 更新全局利息
        accrue_interest(&mut ctx.accounts.pool)?;

        // 2. 更新用户仓位
        let user_position = &mut ctx.accounts.user_position;
        user_position.borrowed_amount = user_position
            .borrowed_amount
            .checked_sub(amount)
            .ok_or(LendingError::MathOverflow)?;

        // 3. 更新资金池
        let pool = &mut ctx.accounts.pool;
        pool.total_borrowed = pool
            .total_borrowed
            .checked_sub(amount)
            .ok_or(LendingError::MathOverflow)?;

        // 4. 转移代币到资金池
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.user_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        // 5. 记录事件
        emit!(LendingEvent {
            event_type: 3, // 3=还款
            amount,
            user: ctx.accounts.user.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn liquidate(
        ctx: Context<Liquidate>,
        repay_amount: u64, // 清算人偿还的债务金额
        seize_amount: u64, // 清算人获取的抵押品金额
    ) -> Result<()> {
        // 1. 更新全局利息
        accrue_interest(&mut ctx.accounts.pool)?;

        // 2. 检查健康因子
        let health_factor = calculate_health_factor(
            ctx.accounts.user_position.deposited_amount,
            ctx.accounts.user_position.borrowed_amount,
            ctx.accounts.pool.collateral_factor,
            get_oracle_price(&ctx.accounts.feed_program,&ctx.accounts.depodit_feed,&ctx.accounts.borrow_feed)?,
        )?;
        require!(health_factor < 100, LendingError::NotLiquidatable);

        // 3. 更新用户仓位
        let user_position = &mut ctx.accounts.user_position;
        user_position.borrowed_amount = user_position
            .borrowed_amount
            .checked_sub(repay_amount)
            .ok_or(LendingError::MathOverflow)?;

        user_position.deposited_amount = user_position
            .deposited_amount
            .checked_sub(seize_amount)
            .ok_or(LendingError::MathOverflow)?;

        // 4. 更新资金池
        let pool = &mut ctx.accounts.pool;
        pool.total_borrowed = pool
            .total_borrowed
            .checked_sub(repay_amount)
            .ok_or(LendingError::MathOverflow)?;

        pool.total_supply = pool
            .total_supply
            .checked_sub(seize_amount)
            .ok_or(LendingError::MathOverflow)?;

        // 5. 转移代币
        // 清算人偿还债务
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.liquidator_token_account.to_account_info(),
                    to: ctx.accounts.pool_token_account.to_account_info(),
                    authority: ctx.accounts.liquidator.to_account_info(),
                },
            ),
            repay_amount,
        )?;

        // 清算人获取抵押品
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.pool_token_account.to_account_info(),
                    to: ctx.accounts.liquidator_token_account.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
            ),
            seize_amount,
        )?;

        // 6. 记录事件
        emit!(LendingEvent {
            event_type: 4, // 4=清算
            amount: repay_amount,
            user: ctx.accounts.user.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }
}
