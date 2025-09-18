use clap::{Arg, Command, value_parser};

const ARG_BIRTHDATE: &str = "birthdate";
const ARG_COUNTRY: &str = "country";
const ARG_DESCRIPTION: &str = "description";
const ARG_EMAIL: &str = "email";
const ARG_FULL_NAME: &str = "full-name";
const ARG_MONTHLY_PRICE_CENTS: &str = "monthly-price-cents";
const ARG_NAME: &str = "name";
const ARG_PASSWORD: &str = "password";
const ARG_PLAN_ID: &str = "plan-id";
const ARG_QUOTA_GIB: &str = "quota-gib";
const ARG_USERNAME: &str = "username";
const ARG_YEARLY_PRICE_CENTS: &str = "yearly-price-cents";

const COMMAND_CREATE_PLAN: &str = "create-plan";
const COMMAND_CREATE_USER: &str = "create-user";
const COMMAND_DISABLE_USER: &str = "disable-user";
const COMMAND_ENABLE_USER: &str = "enable-user";
const COMMAND_LIST_PLANS: &str = "list-plans";
const COMMAND_SET_USER_PLAN: &str = "set-user-plan";

use drive_core::inputs::RegisterInput;
use drive_core::server::commands::{
    disable_user, enable_user, get_all_plans, get_plan_by_id, get_user_by_username, insert_plan, insert_user,
    update_user_plan,
};
use serde_json::to_string_pretty;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    let arg_username = Arg::new(ARG_USERNAME)
        .short('u')
        .long("username")
        .value_parser(value_parser!(String));
    let version = env!("CARGO_PKG_VERSION");
    let matches = Command::new("Mango³ CLI")
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
            Command::new(COMMAND_CREATE_USER)
                .version(version)
                .arg(arg_username.clone())
                .arg(
                    Arg::new(ARG_EMAIL)
                        .short('e')
                        .long("email")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new(ARG_PASSWORD)
                        .short('p')
                        .long("password")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new(ARG_FULL_NAME)
                        .short('n')
                        .long("full-name")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new(ARG_BIRTHDATE)
                        .short('b')
                        .long("birthdate")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new(ARG_COUNTRY)
                        .short('c')
                        .long("country")
                        .value_parser(value_parser!(String)),
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
        )
        .get_matches();

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
        Some((COMMAND_CREATE_USER, matches)) => {
            let username = matches
                .get_one::<String>(ARG_USERNAME)
                .cloned()
                .expect("Could not get argument username");
            let email = matches
                .get_one::<String>(ARG_EMAIL)
                .cloned()
                .expect("Could not get argument email");
            let password = matches
                .get_one::<String>(ARG_PASSWORD)
                .cloned()
                .expect("Could not get argument password");
            let full_name = matches
                .get_one::<String>(ARG_FULL_NAME)
                .cloned()
                .expect("Could not get argument full-name");
            let birthdate = matches
                .get_one::<String>(ARG_BIRTHDATE)
                .cloned()
                .expect("Could not get argument birthdate");
            let country_alpha2 = matches
                .get_one::<String>(ARG_COUNTRY)
                .cloned()
                .expect("Could not get argument country");

            let result = insert_user(&RegisterInput {
                username,
                email,
                password,
                full_name,
                birthdate,
                country_alpha2,
            })
            .await;

            match result {
                Ok(_) => println!("User created successfully."),
                Err(err) => println!("Failed to create user.\n{err}"),
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
        _ => println!("Nothing to do."),
    }
}
