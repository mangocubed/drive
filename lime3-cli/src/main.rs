use clap::{Arg, Command, value_parser};

const ARG_BIRTHDATE: &str = "birthdate";
const ARG_COUNTRY: &str = "country";
const ARG_EMAIL: &str = "email";
const ARG_FULL_NAME: &str = "full-name";
const ARG_HAS_ANNUAL_BILLING: &str = "has-annual-billing";
const ARG_MEMBERSHIP_CODE: &str = "membership-code";
const ARG_PASSWORD: &str = "password";
const ARG_USERNAME: &str = "username";

const COMMAND_CREATE_USER: &str = "create-user";
const COMMAND_DISABLE_USER: &str = "disable-user";
const COMMAND_ENABLE_USER: &str = "enable-user";
const COMMAND_SET_USER_MEMBERSHIP: &str = "set-user-membership";

use lime3_core::inputs::RegisterInput;
use lime3_core::server::commands::{
    disable_user, enable_user, get_user_by_username, insert_user, update_user_membership,
};

#[tokio::main]
async fn main() {
    let arg_username = Arg::new(ARG_USERNAME)
        .short('u')
        .long("username")
        .value_parser(value_parser!(String));
    let version = env!("CARGO_PKG_VERSION");
    let matches = Command::new("Lime3 CLI")
        .version(version)
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
        .subcommand(
            Command::new(COMMAND_SET_USER_MEMBERSHIP)
                .version(version)
                .arg(arg_username)
                .arg(
                    Arg::new(ARG_MEMBERSHIP_CODE)
                        .short('m')
                        .long("membership-code")
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new(ARG_HAS_ANNUAL_BILLING)
                        .short('a')
                        .long("has-annual-billing")
                        .default_value("false")
                        .value_parser(value_parser!(bool)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
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
                .expect("Could not get argument full name");
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
                Ok(_) => {
                    println!("User created successfully.");
                }
                Err(err) => {
                    println!("Failed to create user.\n{err}");
                }
            }
        }
        Some((COMMAND_DISABLE_USER, matches)) => {
            let username = matches
                .get_one::<String>(ARG_USERNAME)
                .expect("argument username is missing");
            let user = get_user_by_username(username).await.expect("could not get user");
            let result = disable_user(&user).await;

            match result {
                Ok(_) => {
                    println!("User disabled successfully.")
                }
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
                Ok(_) => {
                    println!("User enabled successfully.")
                }
                _ => println!("Failed to enable user."),
            }
        }
        Some((COMMAND_SET_USER_MEMBERSHIP, matches)) => {
            let username = matches
                .get_one::<String>(ARG_USERNAME)
                .expect("Argument username is missing");
            let membership_code = matches
                .get_one::<String>(ARG_MEMBERSHIP_CODE)
                .expect("Argument membership-code is missing");
            let has_annual_billing = matches
                .get_one::<bool>(ARG_HAS_ANNUAL_BILLING)
                .expect("Argument has-annual-billing is missing");
            let user = get_user_by_username(username).await.expect("Could not get user");
            let result = update_user_membership(&user, membership_code, *has_annual_billing).await;

            match result {
                Ok(_) => {
                    println!("User membership updated successfully.")
                }
                _ => println!("Failed to update user membership."),
            }
        }
        _ => {
            println!("Nothing to do.");
        }
    }
}
