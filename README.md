# Lime3

Cloud storage & digital distribution platform.

## Environment variables

| Name                          | Type    | Default                                                          |
| ----------------------------- | ------- | ---------------------------------------------------------------- |
| DATABASE_MAX_CONNECTIONS      | Integer | 5                                                                |
| DATABASE_URL                  | String  | postgres://lime3:lime3@127.0.0.1:5432/lime3_dev                  |
| POLAR_BASE_URL                | String  | https://sandbox-api.polar.sh/v1/                                 |
| POLAR_ACCESS_TOKEN            | String  |                                                                  |
| POLAR_SUCCESS_BASE_URL        | String  | http://127.0.0.1:8080/                                           |
| SESSION_DOMAIN                | String  | localhost                                                        |
| SESSION_KEY                   | String  | abcdefghijklmnopqrestuvvwxyz0123456789ABCDEFGHIJKLMNOPQRESTUVVWX |
| SESSION_NAME                  | String  | _lime3_session                                                   |
| SESSION_REDIS_URL             | String  | redis://127.0.0.1:6379/0                                         |
| SESSION_SECURE                | Boolean | false                                                            |
| STORAGE_IMAGE_FILTER_TYPE     | String  | CatmullRom                                                       |
| STORAGE_MAX_SIZE_GIB_PER_FILE | Integer | 1                                                                |
| STORAGE_PATH                  | String  | ./storage                                                        |
| USERS_FREE_QUOTA_GIB          | Integer | 5                                                                |
| USERS_LIMIT                   | Integer | 10                                                               |
