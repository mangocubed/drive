use std::collections::HashMap;

use polar_rs::{
    AmountType, CheckoutSession, CheckoutSessionParams, PriceParams, ProductParams, RecurringInterval,
    SubscriptionParams,
};
use uuid::Uuid;

use crate::server::config::POLAR_CONFIG;
use crate::server::models::{Plan, User};
use crate::server::{POLAR_CLIENT, db_pool};

use super::get_user_by_id;

pub async fn cancel_subscription(user: &User<'_>) -> anyhow::Result<()> {
    let Some(subscription_id) = user.polar_subscription_id else {
        return Err(anyhow::anyhow!("User has no active subscription"));
    };

    let subscription = POLAR_CLIENT
        .update_subscription(
            subscription_id,
            &SubscriptionParams {
                cancel_at_period_end: Some(true),
                ..Default::default()
            },
        )
        .await?;

    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE users SET polar_subscription_id = NULL, plan_expires_at = $2 WHERE id = $1",
        user.id,                         // $1
        subscription.current_period_end  // $2
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn confirm_plan_checkout(checkout_id: Uuid) -> anyhow::Result<()> {
    let checkout = POLAR_CLIENT.get_checkout_session(checkout_id).await?;

    if !checkout.status.is_succeeded() {
        return Err(anyhow::anyhow!("Invalid checkout session"));
    }

    let Some(user_id) = checkout.external_customer_id.and_then(|id| id.parse().ok()) else {
        return Err(anyhow::anyhow!("Invalid user ID"));
    };

    let user = get_user_by_id(user_id).await?;

    let db_pool = db_pool().await;

    let plan = get_plan_by_product_id(checkout.product_id).await?;

    if Some(plan.id) == user.plan_id && checkout.subscription_id == user.polar_subscription_id {
        return Ok(());
    }

    sqlx::query!(
        "UPDATE users SET plan_id = $2, polar_subscription_id = $3 WHERE id = $1",
        user.id,                  // $1
        plan.id,                  // $2
        checkout.subscription_id  // $3
    )
    .execute(db_pool)
    .await?;

    Ok(())
}

pub async fn create_user_plan_checkout(
    user: &User<'_>,
    plan: &Plan<'_>,
    is_yearly: bool,
) -> anyhow::Result<CheckoutSession> {
    let product_id = if is_yearly {
        plan.polar_yearly_product_id
    } else {
        plan.polar_monthly_product_id
    };

    Ok(POLAR_CLIENT
        .create_checkout_session(&CheckoutSessionParams {
            external_customer_id: Some(user.id.to_string()),
            customer_name: Some(user.full_name.to_string()),
            customer_email: Some(user.email.to_string()),
            products: vec![product_id],
            success_url: Some(
                POLAR_CONFIG
                    .success_base_url
                    .join("confirm-checkout?checkout_id={CHECKOUT_ID}")
                    .unwrap(),
            ),
            ..Default::default()
        })
        .await?)
}

pub async fn get_all_plans<'a>() -> sqlx::Result<Vec<Plan<'a>>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(Plan, "SELECT * FROM plans ORDER BY quota_gib ASC")
        .fetch_all(db_pool)
        .await
}

pub async fn get_plan_by_id<'a>(id: Uuid) -> sqlx::Result<Plan<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(Plan, "SELECT * FROM plans WHERE id = $1 LIMIT 1", id)
        .fetch_one(db_pool)
        .await
}

pub async fn get_plan_by_product_id<'a>(product_id: Uuid) -> sqlx::Result<Plan<'a>> {
    let db_pool = db_pool().await;

    sqlx::query_as!(
        Plan,
        "SELECT * FROM plans WHERE polar_monthly_product_id = $1 OR polar_yearly_product_id = $1 LIMIT 1",
        product_id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn insert_plan(
    name: &str,
    description: &str,
    quota_gib: u8,
    monthly_price_cents: u8,
    yearly_price_cents: u16,
) -> sqlx::Result<()> {
    let db_pool = db_pool().await;

    let mut metadata = HashMap::new();

    metadata.insert("quota_gib".to_owned(), quota_gib.to_string());

    let monthly_product = POLAR_CLIENT
        .create_product(&ProductParams {
            name: format!("{name} (monthly subscription)"),
            description: Some(description.to_owned()),
            recurring_interval: Some(RecurringInterval::Month),
            prices: vec![PriceParams {
                amount_type: AmountType::Fixed,
                price_currency: Some("usd".to_owned()),
                price_amount: Some(monthly_price_cents as u32),
                ..Default::default()
            }],
            metadata: metadata.clone(),
            ..Default::default()
        })
        .await
        .expect("Failed to create monthly product");

    let yearly_product = POLAR_CLIENT
        .create_product(&ProductParams {
            name: format!("{name} (yearly subscription)"),
            description: Some(description.to_owned()),
            recurring_interval: Some(RecurringInterval::Year),
            prices: vec![PriceParams {
                amount_type: AmountType::Fixed,
                price_currency: Some("usd".to_owned()),
                price_amount: Some(yearly_price_cents as u32),
                ..Default::default()
            }],
            metadata: metadata.clone(),
            ..Default::default()
        })
        .await
        .expect("Failed to create yearly product");

    sqlx::query!(
        "INSERT INTO plans (
            name,
            description,
            quota_gib,
            monthly_price_cents,
            yearly_price_cents,
            polar_monthly_product_id,
            polar_yearly_product_id
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        name,                       // $1
        description,                // $2
        quota_gib as i16,           // $3
        monthly_price_cents as i16, // $4
        yearly_price_cents as i16,  // $5
        monthly_product.id,         // $6
        yearly_product.id           // $7
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}

pub async fn update_user_plan(user: &User<'_>, plan: &Plan<'_>) -> Result<(), sqlx::Error> {
    let db_pool = db_pool().await;

    sqlx::query!(
        "UPDATE users SET plan_id = $2 WHERE id = $1",
        user.id, // $1
        plan.id  // $2
    )
    .execute(db_pool)
    .await
    .map(|_| ())
}
