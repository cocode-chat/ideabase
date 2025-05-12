### Get a User
```shell
curl --location --request POST 'http://localhost:8080/api/v1/rest/get.json' \
--header 'Content-Type: application/json' \
--data-raw '{
  "timeline.User":{
    "id":38710
  }
}'
```
```json
{
    "code": 0,
    "data": {
        "timeline.User": {
            "avatar": "avatar/head.png",
            "email": "17682301768@qq.com",
            "gender": "m",
            "gmt_create": "2025-05-06 10:18:32",
            "gmt_update": "2025-05-06 10:18:32",
            "id": 38710,
            "nickname": "哈哈哈哈",
            "phone": "17682301768",
            "username": "Zk"
        }
    }
}
```
```sql
# 执行的SQL
SELECT * FROM timeline.user WHERE id=? LIMIT 1 OFFSET 0, params: [38710]
```

### Get Users

```shell
curl --location --request POST 'http://localhost:8080/api/v1/rest/get.json' \
--header 'Content-Type: application/json' \
--data-raw '{
  "[]":{
    "count":2,
    "timeline.User":{
      "@order": "id asc",
      "@column":"id, username"
    }
  }
}'
```
```json
{
    "code": 0,
    "data": {
        "[]": [
            {
                "timeline.User": {
                    "id": 38710,
                    "username": "Zk"
                }
            },
            {
                "timeline.User": {
                    "id": 38711,
                    "username": "Zkk"
                }
            }
        ]
    }
}
```
```sql
# 执行的SQL
SELECT id,username FROM timeline.user ORDER BY id asc LIMIT 2 OFFSET 0
```

### Get Moment with moment's User

```shell
```
```json
```
```sql
# 执行的SQL
SELECT * FROM timeline.moment LIMIT 1 OFFSET 0
SELECT * FROM timeline.user WHERE id=? LIMIT 1 OFFSET 0, params: [38710]
```

### Get Moments with moment's Users

```shell
curl --location --request POST 'http://localhost:8080/api/v1/rest/get.json' \
--header 'Content-Type: application/json' \
--data-raw '{
  "timeline.Moment":{
  },
  "timeline.User":{
    "id@":"timeline.Moment/user_id"
  }
}'
```
```json
{
    "code": 0,
    "data": {
        "timeline.Moment": {
            "comment_ct": 0,
            "content": "第一条朋友圈",
            "gmt_create": "2025-05-06 11:44:55",
            "gmt_update": "2025-05-06 12:00:03",
            "id": 28710,
            "image_urls": [
                "moment/111111.png",
                "moment/111112.png"
            ],
            "like_cnt": 0,
            "location": null,
            "user_id": 38710
        },
        "timeline.User": {
            "avatar": "avatar/head.png",
            "email": "17682301768@qq.com",
            "gender": "m",
            "gmt_create": "2025-05-06 10:18:32",
            "gmt_update": "2025-05-06 10:18:32",
            "id": 38710,
            "nickname": "哈哈哈哈",
            "phone": "17682301768",
            "username": "Zk"
        }
    }
}
```
```sql
# 执行的SQL
SELECT * FROM timeline.moment LIMIT 2 OFFSET 0, params: []
SELECT * FROM timeline.user WHERE id in (?,?) LIMIT 1000 OFFSET 0, params: [38710,38711]
```


### Get Moments with moment's Users

```shell
curl --location --request POST 'http://localhost:8080/api/v1/rest/get.json' \
--header 'Content-Type: application/json' \
--data-raw '{
  "[]": {
    "count": 2,
    "timeline.Moment":{
    },
    "timeline.User":{
      "id@":"[]/timeline.Moment/user_id"
    }
  }
}'
```
```json
{
    "code": 0,
    "data": {
        "[]": [
            {
                "timeline.Moment": {
                    "comment_ct": 0,
                    "content": "第一条朋友圈",
                    "gmt_create": "2025-05-06 11:44:55",
                    "gmt_update": "2025-05-06 12:00:03",
                    "id": 28710,
                    "image_urls": [
                        "moment/111111.png",
                        "moment/111112.png"
                    ],
                    "like_cnt": 0,
                    "location": null,
                    "user_id": 38710
                },
                "timeline.User": {
                    "avatar": "avatar/head.png",
                    "email": "17682301768@qq.com",
                    "gender": "m",
                    "gmt_create": "2025-05-06 10:18:32",
                    "gmt_update": "2025-05-06 10:18:32",
                    "id": 38710,
                    "nickname": "哈哈哈哈",
                    "phone": "17682301768",
                    "username": "Zk"
                }
            },
            {
                "timeline.Moment": {
                    "comment_ct": 0,
                    "content": "this is a show!",
                    "gmt_create": "2025-05-06 11:58:13",
                    "gmt_update": "2025-05-06 12:00:03",
                    "id": 28711,
                    "image_urls": [
                        "moment/111111.png",
                        "moment/111112.png"
                    ],
                    "like_cnt": 0,
                    "location": null,
                    "user_id": 38711
                },
                "timeline.User": {
                    "avatar": "avatar/head.png",
                    "email": "17682301111@qq.com",
                    "gender": "f",
                    "gmt_create": "2025-05-06 10:22:42",
                    "gmt_update": "2025-05-06 10:22:42",
                    "id": 38711,
                    "nickname": "吼吼吼",
                    "phone": "17682301111",
                    "username": "Zkk"
                }
            }
        ]
    }
}
```
```sql
# 执行的SQL
SELECT * FROM timeline.moment LIMIT 2 OFFSET 0, params: []
SELECT * FROM timeline.user WHERE id in (?,?) LIMIT 2 OFFSET 0, params: [38710,38711]
```

### Get Moments with moment's Users and moment's Comments

```shell
curl --location --request POST 'http://localhost:8080/api/v1/rest/get.json' \
--header 'Content-Type: application/json' \
--data-raw '{
  "[]":{                            
    "page":0,                       
    "count":2,
    "timeline.Moment":{               
      "content$":"%a%"              
    },
    "timeline.User":{
      "id@":"[]/timeline.Moment/user_id",  
      "@column":"id,username,avatar"      
    },
    "Comment[]":{                   
      "count":2,
      "timeline.Comment":{
        "moment_id@":"[]/timeline.Moment/id"
      }
    }
  }
}'
```
```json
{
    "code": 0,
    "data": {
        "[]": [
            {
                "Comment[]": [
                    {
                        "timeline.Comment": {
                            "content": "呼呼呼呼",
                            "gmt_create": "2025-05-06 12:03:12",
                            "gmt_update": "2025-05-06 12:03:12",
                            "id": 18711,
                            "like_count": 0,
                            "moment_id": 28711,
                            "parent_comment_id": 0,
                            "user_id": 38710
                        }
                    }
                ],
                "timeline.Moment": {
                    "comment_ct": 0,
                    "content": "this is a show!",
                    "gmt_create": "2025-05-06 11:58:13",
                    "gmt_update": "2025-05-06 12:00:03",
                    "id": 28711,
                    "image_urls": [
                        "moment/111111.png",
                        "moment/111112.png"
                    ],
                    "like_cnt": 0,
                    "location": null,
                    "user_id": 38711
                },
                "timeline.User": {
                    "avatar": "avatar/head.png",
                    "id": 38711,
                    "username": "Zkk"
                }
            }
        ]
    }
}
```
```sql
# 执行的SQL
SELECT * FROM timeline.moment WHERE content LIKE ? LIMIT 2 OFFSET 0, params: ["%a%"]
SELECT id,username,avatar FROM timeline.user WHERE id in (?) LIMIT 1 OFFSET 0, params: [38711]
SELECT * FROM timeline.comment WHERE moment_id in (?) LIMIT 2 OFFSET 0, params: [28711]
```