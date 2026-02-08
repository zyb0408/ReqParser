# ReqParser 测试样例

以下样例可以直接复制粘贴到 ReqParser 的输入框中进行测试。支持三种格式：cURL、fetch、原始 HTTP。

---

## 1. cURL — GET 请求（带查询参数 + Cookie）

```
curl 'https://anyrouter.top/api/log/self/stat?type=0&token_name=&model_name=&start_timestamp=1770480000&end_timestamp=1770536299&group=' \
  -H 'accept: application/json, text/plain, */*' \
  -H 'accept-language: en,zh-CN;q=0.9,zh;q=0.8' \
  -H 'cache-control: no-store' \
  -b 'session=MTc2ODgzNjEw; acw_tc=2ff617a217705315504591037e; cdn_sec_tc=2ff617a217705315504591037e; acw_sc__v2=69882ade282d91973da11de9' \
  -H 'dnt: 1' \
  -H 'new-api-user: 22856' \
  -H 'referer: https://anyrouter.top/console/log' \
  -H 'sec-ch-ua: "Not(A:Brand";v="8", "Chromium";v="144", "Google Chrome";v="144"' \
  -H 'sec-ch-ua-mobile: ?0' \
  -H 'sec-ch-ua-platform: "macOS"' \
  -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/144.0.0.0 Safari/537.36'
```

---

## 2. cURL — POST 请求（带 JSON Body）

```
curl 'https://api.example.com/v1/chat/completions' \
  -X POST \
  -H 'content-type: application/json' \
  -H 'authorization: Bearer sk-1234567890abcdef' \
  -H 'accept: application/json' \
  --data-raw '{"model":"gpt-4","messages":[{"role":"user","content":"hello"}],"temperature":0.7}'
```

---

## 3. fetch — GET 请求（带 Cookie）

```
fetch("https://anyrouter.top/api/user/models", {
  "headers": {
    "accept": "application/json, text/plain, */*",
    "accept-language": "en,zh-CN;q=0.9,zh;q=0.8",
    "cache-control": "no-store",
    "new-api-user": "22856",
    "sec-ch-ua": "\"Not(A:Brand\";v=\"8\", \"Chromium\";v=\"144\", \"Google Chrome\";v=\"144\"",
    "sec-ch-ua-mobile": "?0",
    "sec-ch-ua-platform": "\"macOS\"",
    "cookie": "session=MTc2ODgzNjEw; acw_tc=2ff617a217705315504591037e; cdn_sec_tc=2ff617a217705315504591037e",
    "Referer": "https://anyrouter.top/console/token"
  },
  "body": null,
  "method": "GET"
});
```

---

## 4. fetch — POST 请求（带 Body）

```
fetch("https://api.example.com/v1/messages", {
  "headers": {
    "content-type": "application/json",
    "authorization": "Bearer sk-ant-1234567890",
    "accept": "application/json"
  },
  "body": "{\"model\":\"claude-opus-4-6\",\"max_tokens\":1024,\"messages\":[{\"role\":\"user\",\"content\":\"Hello\"}]}",
  "method": "POST"
});
```

---

## 5. 原始 HTTP — GET 请求

```
GET /api/users?page=2&limit=20&sort=name HTTP/1.1
Host: api.example.com
Accept: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjoiam9obiJ9.abc123
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)
Cache-Control: no-cache
```

---

## 6. 原始 HTTP — 响应（带 Set-Cookie）

```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
Set-Cookie: token=eyJhbGciOiJIUzI1NiJ9; Path=/; HttpOnly; Secure; Max-Age=3600
X-Request-Id: req-9a8b7c6d
Date: Sat, 08 Feb 2026 12:00:00 GMT

{"id": 1, "name": "John Doe", "email": "john@example.com"}
```
