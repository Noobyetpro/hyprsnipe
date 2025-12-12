# hyprsnipe

Rust CLI that checks username availability against the Hytale API using a cookie-authenticated session. **Educational use only; not intended for malicious or unethical activity. Use at your own risk. Licensed under MIT.**

## Prerequisites
- Rust toolchain (stable)
- A working Hytale web session in your browser to copy the session cookie

## Setup
1) Clone or open this repo.
2) Configure `.env`:
```
BASE_URL=https://accounts.hytale.com/api/account/username-reservations/availability?username=
COOKIE=ory_kratos_session=your_cookie_value_here
```
   - `BASE_URL` is the Hytale availability endpoint prefix.
   - `COOKIE` must be the full `Cookie` header value from your logged-in browser session.
3) Prepare `.data.txt` with one username per line to check.

## How to get your Hytale session cookie (Application/Storage tab)
Chrome/Edge (similar on Firefox):
- Log in to accounts.hytale.com.
- Open DevTools → Application  → Cookies and click the `ory_kratos_session`  entry.
- Copy its value. Ignore all other cookies.

## Run
```
cargo run
```
Behavior:
- Reads usernames from `.data.txt`.
- Uses `BASE_URL` + each username, sends your `COOKIE` header.
- Retries on 429 or timeout/connect every 1s until a definitive status.
- Logs `<code>: <status> in <ms> ms`.
- Writes `results.txt` with groups `200:`, `400:`, and `OTHER:` (status shown).

## Notes and ethics
- This is for educational purposes only. Do not use it to harass services, bypass security, or violate terms of use. You are responsible for how you use it.
- Use at your own risk. The authors and contributors assume no liability.
- Licensed under MIT 

