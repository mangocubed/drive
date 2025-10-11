# Mango³ Drive

Cloud storage & digital distribution platform.

## Environment variables

| Name                           | Type    | Default                                           |
| ------------------------------ | ------- | ------------------------------------------------- |
| APP_SERVER_URL                 | String  | http://127.0.0.1:8080/                            |
| APP_OLD_TOKENS                 | Array   | []                                                |
| APP_TOKEN                      | String  |                                                   |
| DATABASE_MAX_CONNECTIONS       | Integer | 5                                                 |
| DATABASE_URL                   | String  | postgres://mango3:mango3@127.0.0.1:5432/drive_dev |
| POLAR_BASE_URL                 | String  | https://sandbox-api.polar.sh/v1/                  |
| POLAR_ACCESS_TOKEN             | String  |                                                   |
| POLAR_SUCCESS_BASE_URL         | String  | http://127.0.0.1:8080/                            |
| STORAGE_FILE_KEY_DURATION_SECS | Integer | 60                                                |
| STORAGE_IMAGE_FILTER_TYPE      | String  | CatmullRom                                        |
| STORAGE_MAX_SIZE_GIB_PER_FILE  | Integer | 1                                                 |
| STORAGE_PATH                   | String  | ./storage                                         |
| USERS_SESSION_TOKEN_LENGTH     | Integer | 32                                                |
| USERS_FREE_QUOTA_GIB           | Integer | 5                                                 |

## Compatibility

| Platform | Status |
| -------- | ------ |
| Web      | ✅     |
| Linux    | ✅     |
| Windows  | ❌     |
| MacOS    | ❌     |
| Android  | ⏳     |
| iOS      | ❌     |
