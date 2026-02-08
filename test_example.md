# ReqParser 测试样例

以下样例可以直接复制粘贴到 ReqParser 的输入框中进行测试。

---

## 1. GET 请求（带查询参数）

```
GET /api/users?page=2&limit=20&sort=name HTTP/1.1
Host: api.example.com
Accept: application/json
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjoiam9obiJ9.abc123
User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)
Cache-Control: no-cache
```

---

## 2. POST 请求（带 JSON Body）

```
POST /api/login HTTP/1.1
Host: api.example.com
Content-Type: application/json
Accept: application/json
X-Request-ID: 8f14e45f-ceea-467f-a830-a]d72
Content-Length: 52

{"username": "admin", "password": "secret123", "remember": true}
```

---

## 3. 带 Cookie 的请求

```
GET /dashboard HTTP/1.1
Host: app.example.com
Cookie: session_id=abc123def456; theme=dark; lang=zh-CN; _ga=GA1.2.123456789
Accept: text/html
Connection: keep-alive
```

---

## 4. HTTP 响应（带 Set-Cookie）

```
HTTP/1.1 200 OK
Content-Type: application/json; charset=utf-8
Set-Cookie: token=eyJhbGciOiJIUzI1NiJ9; Path=/; HttpOnly; Secure; Max-Age=3600
X-Request-Id: req-9a8b7c6d
X-RateLimit-Remaining: 42
Date: Sat, 08 Feb 2026 12:00:00 GMT
Content-Length: 87

{"id": 1, "name": "John Doe", "email": "john@example.com", "role": "admin"}
```

---

## 5. 404 响应

```
HTTP/1.1 404 Not Found
Content-Type: text/html; charset=utf-8
Server: nginx/1.24.0
X-Powered-By: Express
Date: Sat, 08 Feb 2026 12:00:00 GMT

<!DOCTYPE html><html><body><h1>404 Not Found</h1></body></html>
```

---

## 6. 纯 Header（从 DevTools 复制）

```
Content-Type: application/json; charset=utf-8
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.signature
Accept-Language: zh-CN,zh;q=0.9,en;q=0.8
X-Forwarded-For: 192.168.1.100
X-Custom-Header: some-value
```

---

## 7. URL 编码的查询参数

```
GET /search?q=hello%20world&category=%E6%8A%80%E6%9C%AF&tag=rust%2Btauri HTTP/1.1
Host: search.example.com
Accept: */*
```

---

## 8. 多个 Cookie + 复杂 Header

```
GET /api/v2/products HTTP/1.1
Host: store.example.com
Cookie: sid=s%3Aabc123.xyz789; cart_items=3; currency=CNY; _fbp=fb.1.1234567890
Accept: application/json
If-None-Match: "etag-abc123"
Accept-Encoding: gzip, deflate, br
Cache-Control: max-age=0
Referer: https://store.example.com/home
```
