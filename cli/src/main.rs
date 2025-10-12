use clap::{Arg, Command, value_parser};

use drive_core::server::commands::*;

#[cfg(feature = "test-utils")]
use drive_core::test_utils::insert_test_session;

use serde_json::to_string_pretty;
use uuid::Uuid;

const ARG_DESCRIPTION: &str = "description";
const ARG_MONTHLY_PRICE_CENTS: &str = "monthly-price-cents";
const ARG_NAME: &str = "name";
const ARG_PLAN_ID: &str = "plan-id";
const ARG_QUOTA_GIB: &str = "quota-gib";
const ARG_USERNAME: &str = "username";
const ARG_YEARLY_PRICE_CENTS: &str = "yearly-price-cents";

const COMMAND_CREATE_PLAN: &str = "create-plan";
const COMMAND_DISABLE_USER: &str = "disable-user";
const COMMAND_ENABLE_USER: &str = "enable-user";
const COMMAND_LIST_PLANS: &str = "list-plans";
const COMMAND_SET_USER_PLAN: &str = "set-user-plan";

#[cfg(feature = "test-utils")]
const COMMAND_CREATE_TEST_SESSION: &str = "create-test-session";

#[tokio::main]
async fn main() {
    let arg_username = Arg::new(ARG_USERNAME)
        .short('u')
        .long("username")
        .value_parser(value_parser!(String));
    let version = env!("CARGO_PKG_VERSION");
    let command = Command::new("Mango³ CLI")
        .version(version)
        .subcommand(
            Command::new(COMMAND_CREATE_PLAN)
                .version(version)
                .arg(
                    Arg::new(ARG_NAME)
                        .short('n')
                        .long(ARG_NAME)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new(ARG_DESCRIPTION)
                        .short('d')
                        .long(ARG_DESCRIPTION)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new(ARG_QUOTA_GIB)
                        .short('q')
                        .long(ARG_QUOTA_GIB)
                        .value_parser(value_parser!(u8)),
                )
                .arg(
                    Arg::new(ARG_MONTHLY_PRICE_CENTS)
                        .short('m')
                        .long(ARG_MONTHLY_PRICE_CENTS)
                        .value_parser(value_parser!(u8)),
                )
                .arg(
                    Arg::new(ARG_YEARLY_PRICE_CENTS)
                        .short('y')
                        .long(ARG_YEARLY_PRICE_CENTS)
                        .value_parser(value_parser!(u16)),
                ),
        )
        .subcommand(
            Command::new(COMMAND_DISABLE_USER)
                .version(version)
                .arg(arg_username.clone()),
        )
        .subcommand(
            Command::new(COMMAND_ENABLE_USER)
                .version(version)
                .arg(arg_username.clone()),
        )
        .subcommand(Command::new(COMMAND_LIST_PLANS).version(version))
        .subcommand(
            Command::new(COMMAND_SET_USER_PLAN)
                .version(version)
                .arg(arg_username.clone())
                .arg(
                    Arg::new(ARG_PLAN_ID)
                        .short('p')
                        .long(ARG_PLAN_ID)
                        .value_parser(value_parser!(Uuid)),
                ),
        );

    #[cfg(feature = "test-utils")]
    let command = command.subcommand(Command::new(COMMAND_CREATE_TEST_SESSION).version(version));

    let matches = command.get_matches();

    match matches.subcommand() {
        Some((COMMAND_CREATE_PLAN, matches)) => {
            let name = matches
                .get_one::<String>(ARG_NAME)
                .cloned()
                .expect("Could not get argument name");
            let description = matches
                .get_one::<String>(ARG_DESCRIPTION)
                .cloned()
                .expect("Could not get argument description");
            let quota_gib = matches
                .get_one::<u8>(ARG_QUOTA_GIB)
                .cloned()
                .expect("Could not get argument quota-gib");
            let monthly_price_cents = matches
                .get_one::<u8>(ARG_MONTHLY_PRICE_CENTS)
                .cloned()
                .expect("Could not get argument monthly-price-cents");
            let yearly_price_cents = matches
                .get_one::<u16>(ARG_YEARLY_PRICE_CENTS)
                .cloned()
                .expect("Could not get argument yearly-price-cents");

            let result = insert_plan(&name, &description, quota_gib, monthly_price_cents, yearly_price_cents).await;

            match result {
                Ok(_) => println!("Plan created successfully."),
                Err(err) => println!("Failed to create plan.\n{err}"),
            }
        }
        Some((COMMAND_DISABLE_USER, matches)) => {
            let username = matches
                .get_one::<String>(ARG_USERNAME)
                .expect("argument username is missing");
            let user = get_user_by_username(username).await.expect("could not get user");
            let result = disable_user(&user).await;

            match result {
                Ok(_) => println!("User disabled successfully."),
                _ => println!("Failed to disable user."),
            }
        }
        Some((COMMAND_ENABLE_USER, matches)) => {
            let username = matches
                .get_one::<String>(ARG_USERNAME)
                .expect("argument username is missing");
            let user = get_user_by_username(username).await.expect("could not get user");
            let result = enable_user(&user).await;

            match result {
                Ok(_) => println!("User enabled successfully."),
                _ => println!("Failed to enable user."),
            }
        }
        Some((COMMAND_LIST_PLANS, _)) => {
            let result = get_all_plans().await;

            match result {
                Ok(plans) => println!("{}", to_string_pretty(&plans).expect("Failed to serialize plans")),
                _ => println!("Failed to get plans."),
            }
        }
        Some((COMMAND_SET_USER_PLAN, matches)) => {
            let username = matches
                .get_one::<String>(ARG_USERNAME)
                .expect("argument username is missing");
            let plan_id = matches
                .get_one::<Uuid>(ARG_PLAN_ID)
                .expect("argument plan-id is missing");

            let user = get_user_by_username(username).await.expect("Could not get user");
            let plan = get_plan_by_id(*plan_id).await.expect("Could not get plan");

            let result = update_user_plan(&user, &plan).await;

            match result {
                Ok(_) => println!("User plan updated successfully."),
                _ => println!("Failed to update user plan."),
            }
        }
        #[cfg(feature = "test-utils")]
        Some((COMMAND_CREATE_TEST_SESSION, _)) => {
            let session = insert_test_session().await;

            println!("{}", to_string_pretty(&session).expect("Failed to serialize session"));
        }
        _ => println!("Nothing to do."),
    }
}
