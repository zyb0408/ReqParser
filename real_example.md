
## 1.curl

```
curl 'https://anyrouter.top/api/log/self/stat?type=0&token_name=&model_name=&start_timestamp=1770480000&end_timestamp=1770536299&group=' \
  -H 'accept: application/json, text/plain, */*' \
  -H 'accept-language: en,zh-CN;q=0.9,zh;q=0.8' \
  -H 'cache-control: no-store' \
  -b 'session=MTc2ODgzMzYxMHxEWDhFQVFMX2dBQUJFQUVRQUFEX3hQLUFBQVlHYzNSeWFXNW5EQVFBQW1sa0EybHVkQVFFQVA2eWtBWnpkSEpwYm1jTUNnQUlkWE5sY201aGJXVUdjM1J5YVc1bkRBNEFER2RwZEdoMVlsOHlNamcxTmdaemRISnBibWNNQmdBRWNtOXNaUU5wYm5RRUFnQUNCbk4wY21sdVp3d0lBQVp6ZEdGMGRYTURhVzUwQkFJQUFnWnpkSEpwYm1jTUJ3QUZaM0p2ZFhBR2MzUnlhVzVuREFrQUIyUmxabUYxYkhRR2MzUnlhVzVuREEwQUMyOWhkWFJvWDNOMFlYUmxCbk4wY21sdVp3d09BQXhoZWtOYVIwbDBlR1U1VEVjPXykP1Jr8RuM8g4CZXDTAj3Wu6ypzYPFAMWvKxgqOcrNYA==; acw_tc=2ff617a217705315504591037e025a23ce43b4f78a6ffbed1cb2b5e895; cdn_sec_tc=2ff617a217705315504591037e025a23ce43b4f78a6ffbed1cb2b5e895; acw_sc__v2=69882ade282d91973da11de94c3b29212dc3207d' \
  -H 'dnt: 1' \
  -H 'new-api-user: 22856' \
  -H 'priority: u=1, i' \
  -H 'referer: https://anyrouter.top/console/log' \
  -H 'sec-ch-ua: "Not(A:Brand";v="8", "Chromium";v="144", "Google Chrome";v="144"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'sec-fetch-dest: empty' \
  -H 'sec-fetch-mode: cors' \
  -H 'sec-fetch-site: same-origin' \
  -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36'

响应json：
{"data":{"quota":6157893,"rpm":4,"tpm":1665},"message":"","success":true}
```

## 2.fetch
```
fetch请求：
  fetch("https://anyrouter.top/api/user/models", {
  "headers": {
    "accept": "application/json, text/plain, */*",
    "accept-language": "en,zh-CN;q=0.9,zh;q=0.8",
    "cache-control": "no-store",
    "new-api-user": "22856",
    "priority": "u=1, i",
    "sec-ch-ua": "\"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"144\", \"Google Chrome\";v=\"144\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"macOS\"",
    "sec-fetch-dest": "empty",
    "sec-fetch-mode": "cors",
    "sec-fetch-site": "same-origin",
    "cookie": "session=MTc2ODgzMzYxMHxEWDhFQVFMX2dBQUJFQUVRQUFEX3hQLUFBQVlHYzNSeWFXNW5EQVFBQW1sa0EybHVkQVFFQVA2eWtBWnpkSEpwYm1jTUNnQUlkWE5sY201aGJXVUdjM1J5YVc1bkRBNEFER2RwZEdoMVlsOHlNamcxTmdaemRISnBibWNNQmdBRWNtOXNaUU5wYm5RRUFnQUNCbk4wY21sdVp3d0lBQVp6ZEdGMGRYTURhVzUwQkFJQUFnWnpkSEpwYm1jTUJ3QUZaM0p2ZFhBR2MzUnlhVzVuREFrQUIyUmxabUYxYkhRR2MzUnlhVzVuREEwQUMyOWhkWFJvWDNOMFlYUmxCbk4wY21sdVp3d09BQXhoZWtOYVIwbDBlR1U1VEVjPXykP1Jr8RuM8g4CZXDTAj3Wu6ypzYPFAMWvKxgqOcrNYA==; acw_tc=2ff617a217705315504591037e025a23ce43b4f78a6ffbed1cb2b5e895; cdn_sec_tc=2ff617a217705315504591037e025a23ce43b4f78a6ffbed1cb2b5e895; acw_sc__v2=69882ade282d91973da11de94c3b29212dc3207d",
    "Referer": "https://anyrouter.top/console/token"
  },
  "body": null,
  "method": "GET"
});
响应json：
{"data":["claude-3-5-haiku-20241022","claude-3-5-sonnet-20241022","claude-3-7-sonnet-20250219","claude-haiku-4-5-20251001","claude-opus-4-1-20250805","claude-opus-4-20250514","claude-opus-4-5-20251101","claude-opus-4-6","claude-sonnet-4-20250514","claude-sonnet-4-5-20250929","gpt-5-codex"],"message":"","success":true}
```
