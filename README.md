# genesis

[![Deploy](https://github.com/permesi/genesis/actions/workflows/deploy.yml/badge.svg)](https://github.com/permesi/genesis/actions/workflows/deploy.yml)
[![Test & Build](https://github.com/permesi/genesis/actions/workflows/build.yml/badge.svg)](https://github.com/permesi/genesis/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/permesi/genesis/graph/badge.svg?token=KLKV2M5JCT)](https://codecov.io/gh/permesi/genesis)


Token Zero generator

<img src="genesis.svg" height="400">

## Why Ulid?

Helps find(group) tokens for the same period of time but still unique.

```sql
> select id, id::timestamp from tokens;
+----------------------------+-------------------------+
| id                         | id                      |
|----------------------------+-------------------------|
| 01HQAS6A6SGD3Z1V7VF86Q0B6P | 2024-02-23 10:46:47.769 |
| 01HQAS6A6SV2A93NMKH0S03CD1 | 2024-02-23 10:46:47.769 |
| 01HQAS6A6S8ZRMC0RZP8DEQ1Q5 | 2024-02-23 10:46:47.769 |
| 01HQAS6A6S1Q8TT1E8XE1J7JS8 | 2024-02-23 10:46:47.769 |
+----------------------------+-------------------------+

```
