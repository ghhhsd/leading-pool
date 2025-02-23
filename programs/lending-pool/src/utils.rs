
use anchor_lang::prelude::Result;
use crate::{LendingError,LendingPool,Borrow};
use anchor_lang::prelude::*;
// 计算用户健康因子
pub fn calculate_health_factor(
    deposited_amount: u64,
    borrowed_amount: u64,
    collateral_factor: u8,
    price: u64, // 假设 price 是抵押品价格（如 1e6 表示 1.0 USD）
) -> Result<u64> {
    let collateral_value = deposited_amount
        .checked_mul(price)
        .ok_or(LendingError::MathOverflow)?;

    let borrow_limit = collateral_value
        .checked_mul(collateral_factor as u64)
        .ok_or(LendingError::MathOverflow)?
        / 100;

    if borrow_limit == 0 {
        return Ok(0);
    }

    let health_factor = borrow_limit
        .checked_mul(100)
        .ok_or(LendingError::MathOverflow)?
        / borrowed_amount;

    Ok(health_factor)
}

// todo 无预言机，此处先假设一个值
pub fn get_oracle_price(_mint: Pubkey) -> Result<u64> {
    // 从预言机获取价格
    Ok(1_000_000) // 假设价格是 1.0 USD
}

// 借款前的健康检查
pub fn check_before_borrow(ctx: &Context<Borrow>, amount: u64) -> Result<()> {
    let user_position = &ctx.accounts.user_position;
    let pool = &ctx.accounts.pool;

    // 获取抵押品价格（预言机）
    let price = get_oracle_price(pool.mint)?;

    let health_factor = calculate_health_factor(
        user_position.deposited_amount,
        user_position.borrowed_amount + amount,
        pool.collateral_factor,
        price,
    )?;

    require!(health_factor >= 100, LendingError::InsufficientCollateral);

    Ok(())
}


fn calculate_interest_rate(pool: &LendingPool) -> u64 {
    let utilization_rate = (pool.total_borrowed * 100) / pool.total_supply;

    let base_rate = pool.base_rate;
    let slope = match utilization_rate {
        0..=50 => 10,  // 0-50%: 斜率 0.1%
        51..=80 => 20, // 50-80%: 斜率 0.2%
        _ => 30,       // >80%: 斜率 0.3%
    };

    base_rate + (utilization_rate * slope)
}


#[test]
fn test_calculate_interest_rate() {
    let pool = LendingPool {
        total_supply: 100_000,
        total_borrowed: 50_000,
        base_rate: 500, // 5%
        ..LendingPool::default()
    };

    // 资金利用率 = 50%
    let rate = calculate_interest_rate(&pool);
    assert_eq!(rate, 500 + (50 * 10)); // 5% + (50 * 0.2%) = 15%
}

// 更新全局利息
pub fn accrue_interest(pool: &mut Account<LendingPool>) -> Result<()> {
    let current_time = Clock::get()?.unix_timestamp;
    let time_elapsed = current_time - pool.last_update_time;

    if time_elapsed > 0 && pool.total_borrowed > 0 {
        // 计算当前利率
        let borrow_rate = calculate_interest_rate(pool);

        // 计算利息
        let interest = pool.total_borrowed
            .checked_mul(borrow_rate)
            .and_then(|v| v.checked_mul(time_elapsed as u64))
            .ok_or(LendingError::MathOverflow)? / (365 * 24 * 60 * 60); // 年化转秒级

        // 更新总借款（复利）
        pool.total_borrowed = pool.total_borrowed
            .checked_add(interest)
            .ok_or(LendingError::MathOverflow)?;

        // 更新流动性指数和借款指数
        pool.liquidity_index = pool.liquidity_index
            .checked_add((interest * 1_000_000_000 / pool.total_supply).into())
            .ok_or(LendingError::MathOverflow)?;

        pool.borrow_index = pool.borrow_index
            .checked_add((interest * 1_000_000_000 / pool.total_borrowed).into())
            .ok_or(LendingError::MathOverflow)?;

        // 更新最后更新时间
        pool.last_update_time = current_time;
    }
    Ok(())
}



#[macro_export]
macro_rules! err {
    ($error:tt $(,)?) => {
        Err(anchor_lang::error!($crate::utils::ErrorCode::$error))
    };
    ($error:expr $(,)?) => {
        Err(anchor_lang::error!($error))
    };
}