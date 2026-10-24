# Websurfx security audit — “bad” vulnerabilities

**Target:** <https://github.com/neon-mmd/websurfx>  
**Audited commit:** `5353273612f25341040040096b7d4aebcc4b4103` (2026-10-18, package version `1.29.10`)  
**Audit date:** 2026-10-23  
**Method:** manual Rust/JS/config review of the whole `src/`, `public/`, `websurfx/`, Dockerfile and workflows; dependency scan of all 562 `Cargo.lock` packages against OSV/RUSTSEC; local debug **and release** builds run on `127.0.0.1` with curl/unit-test PoCs. No third-party/public instance was tested.

> Severity/CVSS figures are indicative (unauthenticated, network-reachable **public** instance unless a finding says otherwise). Several findings have **no authentication in the product at all**, so “impact” is about users’ browsers, operators’ IP/host, and admin policy — not account takeover.

---

## Executive summary

| # | Finding | Severity | Default build? | Confidence |
|---|---|---|---|---|
| 1 | **One unauthenticated `/search` aborts the release process** (empty fake-useragent list → panic; `panic="abort"`) | **High — CVSS ~7.5** | Yes | **Proven live (SIGABRT)** |
| 2 | **XSS sink: upstream result title/description emitted with `PreEscaped`; result `href` scheme unchecked** (incl. JSON engines Qwant/SepiaSearch; results cached & shared) | **High — ~6.1–7.3** | Sink always; easiest via Qwant/Sepia/third-party instances | **Proven sink (unit test)** |
| 3 | **User-Agent supply chain: cleartext HTTP + invalid-cert acceptance + 2019 EOL HTTP stack + infinite retry / no status or size checks** (`fake-useragent 0.1.3` → `reqwest 0.9`/`hyper 0.12`/`tokio 0.1`) | **High (enabler of #1)** | Yes | Proven from crate source + lockfile |
| 4 | **SafeSearch is effectively broken; level 4 blocklist logic is inverted** | **Medium-High (policy)** | Yes | **Proven live** |
| 5 | **Search query is interpolated unencoded into upstream URLs in 9/11 engines** — parameter/fragment injection, upstream SafeSearch/pagination override | **Medium-High ~6.5** | Yes | Code-proven |
| 6 | **Any client can enable admin-disabled engines** via unsigned `appCookie` or settings import | Medium ~5.3 | Yes | **Proven live (Set-Cookie)** |
| 7 | **Cross-user cache poisoning via ambiguous cache key** (`engines` comma collision) + shared-cache timing/privacy oracle | Medium ~5.3 | Yes (memory/Redis cache) | **Proven live** |
| 8 | **Unauthenticated amplification / resource exhaustion**: 1 search → up to 33–44 upstream fetches, unbounded upstream bodies, unbounded in-memory cache, 50 MiB settings upload, regex compilation | Medium-High | Yes | Code + local upload proof |
| 9 | **ChaCha20-Poly1305 nonce reuse for every cached entry** (`encrypt-cache-results`/`cec-cache-results`) | Medium-High if enabled | Feature-gated | Code-proven (textbook) |
| 10 | **Unsandboxed Lua config = OS command execution** (`os.execute` works) | Medium (trusted-input caveat) | Yes | **Proven live** |
| 11 | **CSRF-able persistent settings import; 1-year cookie with no `SameSite`/`Secure`/`Path`**; importer silently downgrades SafeSearch 2→0 | Medium ~4.3–5.4 | Yes | **Proven live (cross-origin POST)** |
| 12 | Known-vulnerable TLS/HTTP deps in lockfile (`rustls-webpki`/`webpki-root-certs` distrusted emSign, `rustls`, `h2`) | Low-Medium (varies) | Yes | OSV/RUSTSEC |
| 13 | Redirect/SSRF trust boundary, per-worker + peer-IP rate limiting, `allow_any_origin` CORS | Low-Medium | Yes | Code/live headers |
| 14 | Missing security headers, global `Cache-Control: max-age` on searches/`/download`, directory listing, queries in logs, 404 returned as 200 | Low | Yes | **Proven live (headers)** |

**If you only fix three things:** (1) delete `fake-useragent` and use a bundled static UA list + set `panic="unwind"`; (2) stop using `PreEscaped` on upstream data and allowlist `http(s)` result URLs; (3) percent-encode every upstream query with a real URL builder and intersect cookie engines/SafeSearch with server policy.

---

## 1. [HIGH] One unauthenticated search kills the release server

**Where:** `src/user_agent.rs:15-37`, `src/routes/search.rs:170-171`, `Cargo.toml:140` (`panic = "abort"` in `[profile.release]`), `Dockerfile` (builds `--release`).

**Chain:**

1. Every `/search` first calls `random_user_agent()`, which lazily builds a UA list via the `fake-useragent 0.1.3` crate.
2. That crate fetches `http://www.useragentstring.com/pages/useragentstring.php?name=…`, **ignores HTTP status**, and HTML-parses whatever came back. At audit time that host returns `503 Service Temporarily Unavailable`, so parsing yields an **empty vector**.
3. `UserAgents::random()` then does `gen_range(0, self.0.len())` with `len()==0` → `rand 0.6` panics: `Uniform::sample_single called with low >= high`.
4. Debug builds (`panic="unwind"`) lose the worker/request; **release builds abort the entire process**.

**Live proof (release binary, local):**

```text
$ curl http://127.0.0.1:8080/                       # 200
$ curl "http://127.0.0.1:8080/search?q=rust&json=true"
* Empty reply from server
thread 'actix-rt|…' panicked at rand-0.6.5/…/uniform.rs:473:1:
Uniform::sample_single called with low >= high
… Aborted   TMPDIR=… /home/user/target/websurfx-release-unpatched
EXIT_CODE=134                                        # SIGABRT
$ curl http://127.0.0.1:8080/                       # connection refused — server dead
```

Exploitability does not fundamentally depend on today’s 503: *any* condition yielding zero UAs (site redesign, filtering proxy, offline/LAN deployment, or a cleartext MITM — see #3) arms the same crash, and every later search repeats it. A process supervisor restarting the service just resets it for the next single-request kill.

**Fix:** remove the runtime fetch entirely — ship a small static UA list in the binary, pick with a checked RNG, and make “no UA available” a normal error/fallback. Remove `panic="abort"` (or at minimum treat every `unwrap`/index in request paths as a process-fatal bug — see §15). Add a regression test: UA provider returning `[]`, HTTP 503, and connection-refused must never panic or hang.

---

## 2. [HIGH] XSS: upstream titles/descriptions rendered as raw HTML; result URLs not scheme-checked

**Where:** `src/templates/views/search.rs:38-40`

```rust
h1{a href=(result.url){(PreEscaped(&result.title))}}
small{(result.url)}
p{(PreEscaped(&result.description))}
```

`PreEscaped` disables Maud’s escaping. Titles/descriptions come from the network:

- **HTML engines** (DuckDuckGo, Bing, Brave, Searx, LibreX, Mojeek, Startpage, Wikipedia, Yahoo) store `title.inner_html()` / `desc.inner_html()` (`src/engines/*.rs`) — i.e. *serialized markup from the upstream page*, not text. Bing strips only `<strong>`/one `<span>` pattern; the rest strip nothing.
- **JSON engines are worse**: Qwant (`src/engines/qwant.rs:118-130`) and SepiaSearch (`src/engines/sepiasearch.rs:52-68`) copy JSON `title`/`desc`/`name` strings **verbatim** — there is not even the accidental re-escaping an HTML serializer gives text nodes. SepiaSearch indexes user-uploaded PeerTube videos, so an attacker-authored video name/description such as `<img src=x onerror=…>` is a realistic payload source; Qwant titles are attacker-page-controlled text.
- `result.url` is placed in `href` with **no scheme allowlist**, so `javascript:…` / `data:text/html,…` URLs become click-to-XSS links.
- Results are then **cached and shared across users** (`src/routes/search.rs`, §7), so one poisoned result set is re-served until TTL.

**Proven sink (unit test against the real template):** a `SearchResult` with title `<img src=x onerror=alert(1)>`, description `<svg onload=alert(3)>`, url `javascript:alert(2)//https://example.com` renders as:

```html
<a href="javascript:alert(2)//https://example.com"><img src=x onerror=alert(1)></a>
…<p><svg onload=alert(3)></p>
```

**Honest exploitability note:** for *well-behaved* HTML engines that correctly escape webpage titles, `inner_html()` re-serializes the payload as `&lt;img…&gt;`, which displays as text. The bug is that Websurfx **delegates its XSS boundary to every upstream** — including the hardcoded third-party `searx.be` and `search.davidovski.xyz` LibreX instances (`src/engines/searx.rs:62`, `src/engines/librex.rs:68`). A malicious/compromised upstream, a JSON engine, or any upstream that emits rich snippet markup gets script execution in every searching user’s browser, in the instance’s origin. Script there can read/rewrite the non-`HttpOnly` `appCookie`, observe/replace queries and results, phish, and pivot via #11. There is also **no CSP** to blunt it (#14).

**Fix:** treat title/description as *text* — extract `element.text()` (strip tags) in engines and render with normal Maud escaping, never `PreEscaped` on network data. If highlight markup is desired, sanitize with an allowlist sanitizer (e.g. ammonia with only `<b>/<strong>/<em>` and no attributes). Parse result URLs and only allow `http:`/`https:` (drop or neutralize everything else); add `rel="noopener noreferrer"`. Add a CSP as defense-in-depth.

---

## 3. [HIGH] The UA fetcher is a cleartext, cert-ignoring, end-of-life supply chain

**Where:** `Cargo.toml:46` (`fake-useragent = "0.1.3"`), `src/user_agent.rs`; dependency chain proven from `Cargo.lock`: `websurfx → fake-useragent 0.1.3 → reqwest 0.9.24 → hyper 0.12.36 / h2 0.1.26 / tokio 0.1.22 / native-tls`, plus abandoned `failure 0.1.8`, `time 0.1.45`, `idna 0.1.5/0.2.3`, etc.

From the `fake-useragent 0.1.3` crate source itself:

```rust
// http (not https), one thread per browser kind
let url = format!("http://www.useragentstring.com/pages/useragentstring.php?name={}", driver);
let client = reqwest::ClientBuilder::new()
    .danger_accept_invalid_certs(true)
    .danger_accept_invalid_hostnames(true)
    .gzip(true).build().unwrap();
loop { match client.get(url).send() { Ok(mut resp) => return resp.text(), Err(e) => println!(…) } }
```

Consequences:

- **Plain-HTTP fetch on first search** — any on-path attacker fully controls the “UA list” (empty ⇒ #1 crash; huge ⇒ unbounded `resp.text()` memory; crafted strings flow into the `User-Agent` header sent to all upstream engines). `danger_accept_invalid_certs/hostnames(true)` additionally nullifies TLS if it were ever https.
- **Infinite tight retry loop** on connection error (no backoff, timeout, or status check): an unreachable UA host makes the blocking init task spin forever and every `/search` await the same `OnceCell` — a hang-forever DoS for offline/LAN/firewalled deployments.
- The 2019-era stack it drags into the **runtime** binary carries numerous RUSTSEC advisories (e.g. `hyper 0.12.36`: RUSTSEC-2021-0078/0079/2022-0022 — header parsing/request-smuggling class bugs; `h2 0.1.26`: RUSTSEC-2023-0034/2024-0332 DoS; `tokio 0.1.22`: RUSTSEC-2021-0124; `failure`: RUSTSEC-2019-0036/2020-0036; full OSV list in Appendix). Some are only reachable via the already-MITM-able cleartext fetch, which is exactly why the whole dependency should go rather than be risk-accepted line by line.
- It also leaks “a Websurfx instance just started searching” to `useragentstring.com` over HTTP.

**Fix:** delete `fake-useragent` (and with it `reqwest 0.9` entirely). Bundle a few dozen UA strings as a `const &str` array (or a data file) and rotate offline.

---

## 4. [MEDIUM-HIGH] SafeSearch does not work — and level 4 is inverted

Multiple independent breaks; together they mean an operator/user selecting a restrictive level gets little or inverse protection:

1. **Default config is `safe_search = 2` (Moderate) with DuckDuckGo + Wikipedia enabled — both ignore the parameter** (`_safe_search` in `src/engines/duckduckgo.rs:42`, `src/engines/wikipedia.rs:54`). Also ignoring it: Bing, Yahoo, Startpage, LibreX (LibreX hardcodes a `safe_search=on` cookie instead).
2. **Searx zeroes levels 1–2**: `safe_search = u8::from(safe_search >= 3) * 2;` (`src/engines/searx.rs:57`) sends `safesearch=0` upstream for levels 0, 1 **and 2**.
3. **Level ≥3 filtering is only local regex lists that ship empty** (`websurfx/blocklist.txt`, `allowlist.txt` are 0 bytes; `src/aggregator.rs:178-192`). With stock lists, level 3 filters nothing.
4. **Level 4 logic is backwards** (`src/routes/search.rs:304-315`): `flag = !is_match_from_filter_list(blocklist, query); if flag { disallow }` — queries **not** matching the blocklist are disallowed; matching queries proceed. With the shipped empty blocklist, **every** level-4 query is disallowed; with `blockedword` in the list, proven live:

   ```text
   GET /search?q=hello&safesearch=4&json=true        → {"disallowed":true, …}
   GET /search?q=blockedword&safesearch=4&json=true  → {"disallowed":false, …}   ← blocked term allowed
   ```
5. **Settings import downgrades 2→0**: `sanitize()` maps any `safe_search_level >= 2` to `0` (`src/routes/export_import.rs:114-119`). Proven live in §11 (uploaded `2`, cookie set to `0`).
6. Out-of-range values (`safesearch=255` via URL/cookie) are accepted and passed raw to Qwant/upstreams; `get_safesearch_level()` (`src/routes/search.rs:402-412`) only guarantees the config value wins when config ≥ 3 — and even that is undermined upstream by #5 and by engines ignoring the value.

**Fix:** define one server-side mapping per engine and test it (0–2 must reach DDG/Bing/etc. via their real mechanisms, e.g. DDG `kp`, Bing `adlt`); fix the level-4 negation; clamp all inputs to `0..=4`; make import preserve `2`; ship a non-empty default posture or document plainly that 3–4 do nothing without maintained lists.

---

## 5. [MEDIUM-HIGH] Unencoded query injection into upstream search URLs (9 of 11 engines)

**Where:** string interpolation of the raw `query` — `bing.rs:60-62`, `brave.rs:48`, `duckduckgo.rs:54-60`, `librex.rs:68`, `mojeek.rs:114-116`, `searx.rs:62-65`, `startpage.rs:53-56`, `yahoo.rs:132-137`, and Wikipedia via the unencoding `build_query()` (`engines/mod.rs:27-34`, `wikipedia.rs:66-77`). Only Qwant and SepiaSearch use `form_urlencoded::byte_serialize` — proving the correct pattern was known.

Because Actix has already URL-**decoded** `q`, an attacker query containing `&`, `=`, `#`, `+`, `%`, spaces, etc. is spliced into the upstream URL as syntax, e.g. against Searx:

```text
q = cats&safesearch=0#
→ https://searx.be/search?q=cats&safesearch=0#&pageno=2&safesearch=2
                                             └─ everything after # is a fragment, never sent ─┘
```

Impacts: overriding upstream `safesearch`/`safe` even when the operator forced level ≥3 (Mojeek/Searx), truncating pagination/count parameters, injecting engine-specific parameters (Startpage `num=`, LibreX `t=`, DDG `api=`…), cache/upstream result confusion, and plain breakage for legitimate queries containing `&`/`#`/`+`. The same unencoded interpolation exists in the prev/next pagination links (`templates/views/search.rs:112-115`), so a query can smuggle extra `safesearch`/`json`/`page` parameters into the victim’s own next click.

**Fix:** build every upstream URL with `reqwest::Url` + `query_pairs_mut().append_pair("q", query)` (or `form_urlencoded::byte_serialize` everywhere, including `build_query` and pagination links). Add a test with `q="a&b#c=d e+f"` asserting exactly one `q` pair is emitted.

---

## 6. [MEDIUM] Admin-disabled engines can be enabled by any client

**Where:** `src/routes/search.rs:141-159` (cookie parsed with `serde_json::from_str(...).ok()` and used directly; `sanitize()` is **not** called on this path) and `:322-331` (cookie engines filtered only by “engine exists”, via `EngineHandler::new(engine).ok()` — never intersected with `config.upstream_search_engines` **enabled** values). The settings importer has the same flaw: it checks engine names against `upstream_search_engines.keys()` (all known engines), not the enabled ones (`src/routes/export_import.rs:100-112`).

The cookie is unsigned and client-writable by design (even the site’s own JS writes it), so “user preference” silently becomes a server-policy override: an operator who disables e.g. Bing/Searx for legal, privacy, reliability, or content reasons has not actually disabled them.

**Proven live:** with stock config (`Bing = false, Searx = false`), importing `{"engines":["Bing","Searx","DuckDuckGo"],…}` returned `Set-Cookie: appCookie={…"engines":["Bing","Searx","DuckDuckGo"]…}` (§11), and `/search` honors a directly-supplied cookie selecting only a disabled engine.

**Fix:** on every path, `effective_engines = cookie_engines ∩ {engines enabled in config}`; decide explicitly whether users may *subset* but never *expand* the operator’s set. Apply the same intersection in `sanitize()`.

---

## 7. [MEDIUM] Shared-cache poisoning via ambiguous cache key; cross-user privacy oracle

**Where:** `src/routes/search.rs:186-210`. The key is `blake3("http://{ip}:{port}/search?q={query}&page={page}&safesearch={lvl}&engines={engines.join(",")}")` built from **raw, unescaped** fields and the **raw cookie engine strings**.

Since `join(",")` cannot distinguish one engine name containing a comma from two names, these collide:

```text
attacker cookie engines: ["DuckDuckGo,Bing"]   (one bogus name → no real engine runs → empty "filtered" result)
victim   cookie engines: ["DuckDuckGo","Bing"]  (two real engines)
both keys: …&engines=DuckDuckGo,Bing
```

**Proven live:** after the attacker request (79 ms, `{"results":[],"filtered":true}`), the victim request for the same query returned the identical empty/filtered body in **1.9 ms** — a cache hit; no upstream engine ran. An attacker can thus suppress/poison results for chosen queries for the whole instance until TTL (default 600 s, operator-configurable upward). Empty/error result sets are cached too, so transient upstream failures are similarly “pinned” for everyone.

Related privacy issues with the same design: cache hit/miss timing reveals whether *another user* searched the same (query, page, engines, level) within the TTL, and unsalted fast Blake3 keys over a guessable URL template are dictionary-attackable by anyone who can read Redis/memory dumps.

**Fix:** hash a canonical, unambiguous serialization (length-prefixed fields or postcard/JSON of a struct with the **validated, sorted** engine list — not raw cookie strings); don’t cache error/empty-engine states; consider per-instance HMAC keys (`blake3::keyed_hash`) and document the timing-oracle tradeoff for public instances.

---

## 8. [MEDIUM-HIGH] Unauthenticated amplification & resource exhaustion

Several multiplicative factors, all reachable without any account:

- **Prefetch amplification:** one `/search` computes current + previous + next pages (`search.rs:180-270`). Each page fans out to every selected engine (up to 11 via §6) ⇒ **33 upstream fetches per user search** (and **44** on the cache-exists-error path, which refetches the current page too, `search.rs:259-270`). TF-IDF relevance scoring runs over all of it. This burns operator bandwidth/CPU and hammers upstreams from the operator’s IP (ban/reputation risk) — the public JSON API (`?json=true`) exposes the same power with no key.
- **No size caps anywhere:** no limit on `q` length or `page` (u32; see §15 overflow), and upstream bodies are read fully into memory via `.text()` / `.bytes().to_vec()` (`src/models/engine.rs:76-124`) with reqwest gzip/brotli auto-decompression — a malicious/compromised upstream (or HTTP/2 `h2` issue, §12) can return a decompression bomb per engine task, concurrently. Only the 30 s timeout bounds damage.
- **Unbounded in-memory cache:** `MokaCache::builder().time_to_live(…).build()` (`src/cache/memory.rs:32-40`) sets no `max_capacity`/weigher; in the resolved `moka 0.12.16` the builder default is `max_capacity: None` (admit-all) — growth is bounded only by TTL turnover, so unique-query floods (esp. multi-IP/IPv6) grow resident memory until OOM.
- **50 MiB “JSON settings” upload:** no `MultipartFormConfig`/field limit is set, so actix-multipart’s default 50 MiB total applies (verified in resolved `actix-multipart 0.8.5` source) for a file that should be ~200 bytes; the handler then synchronously `read_to_string`s it on an async worker (`src/routes/export_import.rs:147`, blocking I/O) and JSON-parses it. A 5 MiB upload was accepted live with HTTP 200.
- **Attacker-triggerable regex work:** choosing `safesearch=3/4` forces per-request (×3 pages) reading and `Regex::new` compilation of every blocklist/allowlist line, matched against every result (`aggregator.rs:213-254`, `search.rs:373-385`). The bigger the operator’s lists, the better the free CPU amplifier.

**Fix:** cap `q` (≈512 chars) and `page` (≈≤ 50), stream upstream bodies with a hard byte cap (few MiB) and decompression limit, set Moka `max_capacity` with a byte weigher, set `#[multipart(limit = "16 KiB")]` and read the tempfile asynchronously, compile filter regexes once at startup into a `RegexSet`, and make prefetch opt-in/bounded (or drop prev-page prefetch). Put the JSON API behind the same (or stricter) quotas.

---

## 9. [MEDIUM-HIGH, feature-gated] ChaCha20-Poly1305 key+nonce reuse encrypts the whole cache

**Where:** `src/cache/mod.rs:108-135`, `src/cache/encryption.rs`. With `encrypt-cache-results` or `cec-cache-results`:

```rust
let cipher = CIPHER.get_or_init(|| ChaCha20Poly1305::new(&Key::generate()));
let encryption_key = ENCRYPTION_KEY.get_or_init(Nonce::generate); // misnamed: this is the NONCE
cipher.encrypt(encryption_key, bytes) // same (key, nonce) for every entry, forever
```

Reusing a ChaCha20-Poly1305 (key, nonce) pair across messages is catastrophic: ciphertext XORs leak plaintext XORs (here plaintexts are postcard-serialized `SearchResults` with large known structure ⇒ practical keystream recovery → decrypt all entries), and Poly1305 one-time-key reuse destroys authenticity (forgery → crafted cache entries → feeds the §2 XSS/cache-poisoning chain). The random key is also generated per-process and never persisted/shared, so Redis entries become undecryptable after restart and between replicas — the feature provides neither confidentiality nor a working shared cache. (Plain, non-encrypted cache mode instead stores postcard bytes in Redis with no integrity at all; anyone with Redis write access can inject arbitrary `SearchResults` — keep Redis private/authenticated, see §16.)

**Fix:** generate a fresh random nonce per entry, store `nonce ‖ ciphertext`, derive the key from an operator-supplied secret (env/file, KDF), and version the format. Or drop the feature and rely on Redis ACLs/TLS at rest.

---

## 10. [MEDIUM, trusted-input] Config file is unsandboxed Lua — `os.execute` runs at startup

**Where:** `src/parser.rs:78-83`:

```rust
let lua = Lua::new(); // mlua ALL_SAFE still includes io/os/package
lua.load(&fs::read_to_string(file_path(FileType::Config).await?)?).exec_async().await?;
```

**Proven live:** prepending `os.execute("touch /home/user/target/lua-pwned")` to `config.lua` created the file as the server user before the HTTP listener even finished starting. mlua’s docs distinguish the safe subset loadable via `Lua::new_with(StdLib, …)` [1](https://docs.rs/mlua/latest/mlua/struct.Lua.html); `Lua::new()` does not remove `io`/`os`.

This is **not** a remote bug by itself — config is operator-supplied — but it means: never run with a config you didn’t author (copied “example configs”, writable `$HOME/.config/websurfx/config.lua` / `/etc/xdg/websurfx/` / `./websurfx/config.lua` — the search order in `src/handler.rs:63-115`), and treat any future “import config/theme” feature as RCE. A malformed config also panics at startup via `rate_limiter["number_of_requests"]` HashMap indexing (`parser.rs:173-174`) — with release `panic="abort"`, a hard crash rather than an error message.

**Fix:** `Lua::new_with(StdLib::TABLE|STRING|MATH, …)` (no `IO`/`OS`/`PACKAGE`/`DEBUG`), clear `dofile/loadfile/require`, add memory/instruction limits, and validate all extracted values with range checks instead of indexing.

---

## 11. [MEDIUM] CSRF-able settings import sets a flag-less 1-year cookie

**Where:** `POST /settings` (`src/routes/export_import.rs:134-172`). No CSRF token, no `Origin`/`Referer`/`Sec-Fetch-Site` check, no auth. **Proven live** — a cross-origin multipart POST was accepted and set a year-long cookie with **no `HttpOnly`, `SameSite`, `Secure`, or `Path`** (the site’s own JS setter in `public/static/settings.js:78` also sets none):

```http
POST /settings            Origin: https://evil.example   (multipart file=attacker.json)
→ HTTP/1.1 200 OK
  set-cookie: appCookie={"animation":null,"colorscheme":"dracula","engines":["Bing","Searx","DuckDuckGo"],"safe_search_level":0,"theme":"simple"}; Expires=Fri, 22 Oct 2027 …
  access-control-allow-origin: https://evil.example
```

Note the twofer in that single response: disabled engines **Bing/Searx** were enabled (§6) and uploaded `safe_search_level: 2` came back as **`0`** (§4.5). A drive-by page (classic form POST) can persistently retune a victim’s engines/SafeSearch/theme for a year; combined with §2 (enable SepiaSearch, then the victim searches the attacker’s term) it becomes an XSS delivery chain. Caveat: modern browsers may refuse the `Set-Cookie` in some cross-site subresource contexts; top-level form navigations and same-site/subdomain attackers are not so protected, and the server performs no check of its own in any case. `/download` likewise trusts and reflects the unsigned cookie.

**Fix:** require a CSRF token (or strict same-origin `Sec-Fetch-Site`/`Origin` validation) on `POST /settings`; set `SameSite=Lax; Secure` (on HTTPS) `; Path=/` (it cannot be `HttpOnly` while current JS reads it — another reason to move preferences server-side or sign them); prefer HMAC-signing the cookie or treating it as purely cosmetic (never for engine/SafeSearch authorization — see §6).

---

## 12. [LOW-MEDIUM] Current TLS/HTTP dependencies with published advisories

From the OSV batch over `Cargo.lock` (27 affected package-versions total; the old `fake-useragent` tree is §3). Runtime-relevant now:

| Package (locked) | Advisory | Meaning for Websurfx | Fixed |
|---|---|---|---|
| `rustls-webpki 0.103.13` | RUSTSEC-2026-0382 / CVE-2026-111943 | May accept cert paths via **distrusted emSign intermediates whose private keys are well known** → MITM of upstream engine TLS | `0.103.16` |
| `webpki-root-certs 1.0.9` (direct dep) | RUSTSEC-2026-0383 | Bundles the distrusted emSign root (used when `operating_system_tls_certificates=false`) | `1.0.10` |
| `rustls 0.23.40` | RUSTSEC-2026-0285, -0346, -0348, -0349 | TLS1.3 message-boundary laxness (low), session-secret `Debug` exposure (log-dependent), Brotli cert-decompression 1 GiB allocation (feature-dependent), unauthenticated ServerHello key-exchange downgrade | `0.23.45/46` |
| `h2 0.4.14` (via reqwest/hyper) | RUSTSEC-2026-0258 | Unbounded empty DATA frames → memory growth from a malicious HTTP/2 upstream | `0.4.16` |
| `anyhow 1.0.102`, `event-listener 5.4.1`, `ctutils 0.4.3` | RUSTSEC-2026-0190/-0221/-0385 | Soundness/constant-time issues; exploitability in this app appears low/indirect | next patch releases |

**Fix:** `cargo update` the TLS stack now (especially the emSign pair), add `cargo audit`/`cargo deny` to CI, and let Dependabot actually merge. The deeper lesson is §3: an unmaintained leaf crate pinned the project to a second, ancient TLS/HTTP stack (`native-tls`+`hyper 0.12`) that no audit of “main” deps would notice without lockfile-wide scanning.

---

## 13. [LOW-MEDIUM] Redirect trust, rate-limiter scoping, and permissive CORS

- **Upstream redirects/SSRF surface:** the aggregator client (`src/aggregator.rs:84-109`) sets no redirect policy, so reqwest’s default of following up to 10 redirects applies [1](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html). `https_only(true)` (good) blocks http downgrades, but a compromised/malicious upstream can still 30x the server to other HTTPS hosts/IP literals (cloud metadata over https, internal services with valid certs); responses are parsed as result HTML/JSON (blind SSRF) and Websurfx validates neither the final host nor resolved private IPs. Prefer `redirect::Policy::none()` (search endpoints don’t need redirects) or a custom same-host policy; block private/loopback/link-local resolutions if redirects are ever followed.
- **Rate limiter is per-worker and peer-IP based:** `GovernorConfigBuilder…finish()` is called **inside** the `HttpServer::new` app factory (`src/lib.rs:83-89`) and `finish()` creates a fresh `Arc<RateLimiter>` each call ⇒ with `threads = 10` workers the effective quota is up to **10×** the configured one (new connections land on different workers). Separately, actix-governor’s default peer-IP keying [1](https://docs.rs/actix-governor/latest/actix_governor/) means behind a same-host reverse proxy **all users share one bucket** (one abuser DoSes everyone) while direct deployments get no benefit from `X-Forwarded-For`. The config semantics also don’t match the implementation: “20 requests within 3 s” is wired as `seconds_per_request(3)` + `burst_size(20)` (burst 20, then 1 token / 3 s) [1](https://docs.rs/actix-governor/latest/actix_governor/). Build the config **once** outside the factory and use a trusted-proxy-aware extractor when proxied.
- **CORS `allow_any_origin` for GET** (`src/lib.rs:67-75`), reflected live as `access-control-allow-origin: https://evil.example`: any website can read non-credentialed `/search?json=true` and `/download` responses, i.e. use visitors’ browsers/packages as free search-API clients. `ALLOWED headers` listing `Cookie` is ineffective (browsers forbid setting it) and there is no `Access-Control-Allow-Credentials`, which is the only reason credentialed reads fail — don’t “fix” that part by enabling credentials with a wildcard. For a browser UI, drop CORS entirely or allowlist the instance’s own origin; also consider Private-Network-Access implications for localhost instances (a public page probing `http://127.0.0.1:8080`).

---

## 14. [LOW] Browser/cache/privacy hardening gaps (all observed live)

- **No security headers at all** on HTML/JSON: no `Content-Security-Policy` (would substantially blunt §2), `X-Content-Type-Options`, `Referrer-Policy`, `X-Frame-Options`/`frame-ancestors` (settings/search are frameable — clickjacking), `Permissions-Policy`, or HSTS guidance. Result links also lack `rel="noreferrer"`.
- **Global `Cache-Control: max-age=60`** (configurable, minimum-forced 60) is added to *every* response via `DefaultHeaders` (`src/lib.rs:90-94`) — including search-result pages, the JSON API, `/download` (preferences), errors, and the `POST /settings` response. Shared/browser caches may retain query-bearing pages; for a privacy search engine, searches should be `no-store`.
- **Directory listing enabled** on `/static`, `/favicon`, `/images` via `.show_files_listing()` (`src/lib.rs:96-107`) — proven: `GET /static/` returns “Index of /static/”. Remove it.
- **404 handler returns HTTP 200** (`src/routes/mod.rs:29-41`) — cache/index confusion and sloppy error semantics; return `404`.
- **Queries are logged in full** by `Logger::default()` (`src/lib.rs:79`) at INFO when `logging=true` (default): process/Docker logs become a search-history database (`"GET /search?q=…"` seen verbatim in audit logs). Log the route without the query string, or default query logging off.

---

## 15. Systemic note: `panic = "abort"` turns every stray panic into a total outage

§1 is the headline, but the pattern repeats and should be hunted as a class:

- `src/engines/bing.rs:112`: `url.value().attr("href").unwrap()` — an upstream `.tpcn a.tilk` anchor without `href` (valid HTML, upstream-controlled) panics inside a `JoinSet` task; in release that’s an abort, not a per-engine error.
- `templates/partials/settings_tabs/engines.rs`: `&engine_name[..1]` / `[1..]` panics on empty or multi-byte-first-char engine names from `config.lua` — `/settings` becomes a crash button after an admin adds such an engine.
- Unbounded `page`: proven debug panic `attempt to add with overflow` at `templates/views/search.rs:115` for `page=4294967295` (release wraps instead — still wrong). Engine offset math (`page * 30`, `page * 10`, …) is unchecked everywhere except Qwant’s `checked_mul`.
- Config/startup: `rate_limiter[…]` indexing (§10), governor `finish().unwrap()` on zero `time_limit`/`number_of_requests` (`lib.rs:85-88`), `cb.build().unwrap()` (`aggregator.rs:108`).

**Fix:** remove `panic="abort"` for the server profile (the space saving isn’t worth process-wide fragility), forbid `unwrap` in request paths by lint (the crate already sets `clippy::panic = forbid` — extend it / audit the escapes), use `checked_*`/`saturating_*` for pagination, and clamp `page`.

## 16. Deployment notes worth fixing while you’re there

- **Docker final image runs as root** (`FROM scratch`, no `USER`) with the config volume at `/etc/xdg/websurfx/` — add a non-root user and mount config read-only. The app speaks plain HTTP only; docs’ Docker example sets `binding_ip = "0.0.0.0"` — if an operator publishes `8080` directly, queries and the flag-less cookie cross the network in cleartext. Document “always behind a TLS reverse proxy” (and then revisit §13’s peer-IP limiter).
- **Redis examples are unauthenticated** (`redis-server --port 8082 &`, `image: redis:latest` commented without auth). In non-encrypted mode Redis values are bare postcard `SearchResults` deserialized on read (`cache/mod.rs` `TryInto`) — anyone with Redis write gets stored-result injection (→ §2 XSS for all users). Bind Redis to localhost/container-net only, require AUTH/ACL (+TLS if remote), and treat cache contents as untrusted (they’re re-rendered!).
- Config-name drift: sample `config.lua` defines `https_adaptive_window_size` but the parser reads `adaptive_window` (`parser.rs:89`) — a stock checkout may fail to start until one is renamed (the audit harness had to add the alias). Not a vuln, but it suggests the shipped config isn’t smoke-tested — add a startup test.

---

## What was checked and *not* found

- No SQL/database, no shell-outs, no `unsafe` in app code (`unsafe_code = "forbid"`); no user-controlled filesystem paths (theme/config/list paths are fixed or admin-config) — no credible SQLi/cmdi/path-traversal in app code. `actix-files` handles traversal; the exposure there is only the listing (§14).
- The search query `q` itself is correctly escaped by Maud in the HTML body/search box — **no direct reflected XSS via `q`** was found; §2 is about upstream/result data, not `q` echo. Client JS writes user data via `.value`/`.innerText` (no `innerHTML` sinks found); the cookie-driven CSS `href` rewriting in `settings.js` stays same-origin-relative and was not countenanced as XSS.
- Upstream fetching uses rustls with `https_only(true)`; no `danger_accept_invalid_certs` exists in *app* code (it’s inside the §3 dependency). No open-redirect parameter was found (the only redirect is a fixed `/` bounce for empty queries).
- No auth/session system exists, so auth-bypass/IDOR classes don’t apply; the findings above are framed against the actual trust boundaries (user browser ↔ instance ↔ upstream ↔ operator).

## Suggested remediation order

**Today:** remove `fake-useragent` + static UA list; drop `panic="abort"`; update `rustls-webpki`/`webpki-root-certs`/`rustls`/`h2`; take `SepiaSearch`/`Qwant` (or raw-HTML rendering) offline until §2 is fixed.
**This week:** escape/sanitize result text + URL scheme allowlist + CSP; URL-encode all upstream queries; intersect cookie engines/SafeSearch with config; fix SafeSearch mapping + level-4 negation; canonical cache keys; multipart/body/cache caps.
**This month:** CSRF protection + cookie flags/signing; rebuild the rate limiter once with proxy-aware keys; nonce-per-entry cache encryption (or remove); sandbox Lua; `cargo audit` in CI; security headers/`no-store`; regression tests for every PoC above.

## Reproduction notes

All live proofs were against a local instance built from commit `5353273` (the only changes in the harness copy were: adding `adaptive_window = true` to the harness `config.lua` — §16 — and, **only for the §4/§7 probes after §1 was proven**, stubbing `random_user_agent()` to a constant so `/search` could run at all; §1/§2/§10/§11 proofs used unmodified upstream logic/binaries as described). The original clone in the workspace (`websurfx/`) is untouched (`git status` clean). Please don’t run these PoCs against public instances you don’t own — §1 in particular is a one-request kill. The project’s `SECURITY.md` invites responsible reports with reproduction steps; this file is written to be attachable as-is.

### Appendix: OSV-affected locked packages (abridged)

`anyhow 1.0.102` (RUSTSEC-2026-0190), `crossbeam-utils 0.7.2`, `ctutils 0.4.3`, `event-listener 5.4.1`, `failure 0.1.8`, `h2 0.1.26 & 0.4.14`, `hyper 0.12.36`, `idna 0.1.5 & 0.2.3`, `lock_api 0.3.4`, `memoffset 0.5.6`, `net2 0.2.39`, `rand_os 0.1.3`, `rustls 0.23.40`, `rustls-webpki 0.103.13`, `time 0.1.45`, `tokio 0.1.22` (+ `tokio-current-thread/executor/io/reactor/sync/tcp/threadpool/timer 0.1–0.2`), `webpki-root-certs 1.0.9` — 27 package-versions with ≥1 advisory at audit time. Everything ≤2019 in this list enters **only** through `fake-useragent` (§3) and disappears with it.
