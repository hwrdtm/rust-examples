# What

Behavior:

- If the main future finishes first, then return that result without firing off the other future.
- If the main future does not finish before the delay, fire off the other future and return the result of the future that finishes first.
- Handle aborting the either future when the other one finishes.

# Example Logs

```log
[2023-09-20T00:40:53Z DEBUG reqwest::connect] starting new connection: https://www.google.com/
[2023-09-20T00:40:53Z DEBUG delayed_spray_and_pray] [main_task] took longer than 500ms, firing off [backup_task]
[2023-09-20T00:40:53Z DEBUG reqwest::connect] starting new connection: https://google.com/
[2023-09-20T00:40:54Z DEBUG delayed_spray_and_pray] [main_task] finished
Response { url: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("www.google.com")), port: None, path: "/", query: None, fragment: None }, status: 200, headers: {"date": "Wed, 20 Sep 2023 00:40:54 GMT", "expires": "-1", "cache-control": "private, max-age=0", "content-type": "text/html; charset=ISO-8859-1", "content-security-policy-report-only": "object-src 'none';base-uri 'self';script-src 'nonce-hi-3N1M040JaH4DUmSXBCw' 'strict-dynamic' 'report-sample' 'unsafe-eval' 'unsafe-inline' https: http:;report-uri https://csp.withgoogle.com/csp/gws/other-hp", "p3p": "CP=\"This is not a P3P policy! See g.co/p3phelp for more info.\"", "server": "gws", "x-xss-protection": "0", "x-frame-options": "SAMEORIGIN", "set-cookie": "1P_JAR=2023-09-20-00; expires=Fri, 20-Oct-2023 00:40:54 GMT; path=/; domain=.google.com; Secure", "set-cookie": "AEC=Ad49MVFp3zi7z_Lrsx0QZz2x5fBg7AUXR_WNsBoowwe-vpalmF566HO3zwA; expires=Mon, 18-Mar-2024 00:40:54 GMT; path=/; domain=.google.com; Secure; HttpOnly; SameSite=lax", "set-cookie": "NID=511=PHPeOkEjud6YM5Tu-ktK9e2hOGyrCIPbrbgpDQMldB3y1oKR5-F4NfwbgEMzlrV9-rPHx6BZSoPj3jyG-9CgfiOjpaefbppbc_RCfI10q6djm2fUSd1gfqc6IG__s7bQUqYfkNtq0LYCYUFOgo5wNJB7ZW5NTrgqzo7_22w8UZA; expires=Thu, 21-Mar-2024 00:40:54 GMT; path=/; domain=.google.com; HttpOnly", "alt-svc": "h3=\":443\"; ma=2592000,h3-29=\":443\"; ma=2592000", "accept-ranges": "none", "vary": "Accept-Encoding", "transfer-encoding": "chunked"} }
[2023-09-20T00:40:54Z INFO  delayed_spray_and_pray] Starting long running task
[2023-09-20T00:40:58Z DEBUG delayed_spray_and_pray] [main_task] finished
[2023-09-20T00:40:58Z INFO  delayed_spray_and_pray] Starting long running task
```