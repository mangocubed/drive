use clap::{Arg, Command, value_parser};

const ARG_USERNAME: &str = "username";
const ARG_EMAIL: &str = "email";
const ARG_PASSWORD: &str = "password";
const ARG_FULL_NAME: &str = "full-name";
const ARG_BIRTHDATE: &str = "birthdate";
const ARG_COUNTRY: &str = "country";

const COMMAND_CREATE_USER: &str = "create-user";

use lime3_core::inputs::RegisterInput;
use lime3_core::server::commands::insert_user;

#[tokio::main]
async fn main() {
    let version = env!("CARGO_PKG_VERSION");
    let matches = Command::new("Lime3 CLI")
        .version(version)
        .subcommand(
            Command::new(COMMAND_CREATE_USER)
                .version(version)
                .arg(
                    Arg::new(ARG_USERNAME)
                        .short('u')
                        .long("username")
                        .value_parser(value_parser!(String)),
                )
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
        _ => {
            println!("Nothing to do.");
        }
    }
}
