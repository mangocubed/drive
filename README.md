# Lime3

Cloud storage & digital distribution platform.

## Environment variables

| Name                     | Type    | Default                                                          |
| ------------------------ | ------- | ---------------------------------------------------------------- |
| BILLING_POLAR_BASE_URL   | String  | https://sandbox-api.polar.sh/v1/                                 |
| BILLING_POLAR_TOKEN      | String  |                                                                  |
| BILLING_SUCCESS_BASE_URL | String  | http://127.0.0.1:8080/                                           |
| DATABASE_MAX_CONNECTIONS | Integer | 5                                                                |
| DATABASE_URL             | String  | postgres://lime3:lime3@127.0.0.1:5432/lime3_dev                  |
| SESSION_DOMAIN           | String  | localhost                                                        |
| SESSION_KEY              | String  | abcdefghijklmnopqrestuvvwxyz0123456789ABCDEFGHIJKLMNOPQRESTUVVWX |
| SESSION_NAME             | String  | _lime3_session                                                   |
| SESSION_REDIS_URL        | String  | redis://127.0.0.1:6379/0                                         |
| SESSION_SECURE           | Boolean | false                                                            |
| STORAGE_PATH             | String  | ./storage                                                        |
