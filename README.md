# Lime3

Cloud storage & digital distribution platform.

## Environment variables

| Name                      | Type    | Default                                                          |
| ------------------------- | ------- | ---------------------------------------------------------------- |
| DATABASE_MAX_CONNECTIONS  | Integer | 5                                                                |
| DATABASE_URL              | String  | postgres://lime3:lime3@127.0.0.1:5432/lime3_dev                  |
| PRICING_DEFAULT_QUOTA     | String  | 1 GiB                                                            |
| PRICING_FREE_QUOTA        | String  | 1 GiB                                                            |
| PRICING_MAX_QUOTA         | String  | 10 GiB                                                           |
| SESSION_DOMAIN            | String  | localhost                                                        |
| SESSION_KEY               | String  | abcdefghijklmnopqrestuvvwxyz0123456789ABCDEFGHIJKLMNOPQRESTUVVWX |
| SESSION_NAME              | String  | _lime3_session                                                   |
| SESSION_REDIS_URL         | String  | redis://127.0.0.1:6379/0                                         |
| SESSION_SECURE            | Boolean | false                                                            |
| STORAGE_IMAGE_FILTER_TYPE | String  | CatmullRom                                                       |
| STORAGE_MAX_SIZE_PER_FILE | String  | 100 MiB                                                          |
| STORAGE_PATH              | String  | ./storage                                                        |
