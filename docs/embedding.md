# Embedding Websurfx in a Rust application

Websurfx already builds as a library. Its public API exposes `parser::Config` and `run`, so a host
application can configure and start the Websurfx server without launching the `websurfx` binary.
The `embedded` feature adds a method for selecting the resource directory used by that server.

## Install

Add Websurfx and Tokio to the host application's `Cargo.toml`:

```toml
[dependencies]
websurfx = { git = "https://github.com/neon-mmd/websurfx", branch = "rolling", features = ["embedded"] }
tokio = { version = "1", features = ["macros", "net", "rt-multi-thread", "sync"] }
```

Until the feature is merged upstream, test this draft with the PR branch instead:

```toml
websurfx = { git = "https://github.com/undivisible/websurfx", branch = "feature/embedded-library", features = ["embedded"] }
```

## Supply the runtime data

The `embedded` feature does not compile Websurfx's data files into the executable. The host
application supplies a directory at runtime. Its layout is:

```text
websurfx-data/
├── public/
│   ├── favicon/
│   ├── images/
│   ├── static/
│   ├── robots.txt
│   └── websurfx.xml
└── websurfx/
    ├── allowlist.txt
    ├── blocklist.txt
    └── config.lua
```

Install or copy this directory alongside the host application's other runtime data. Websurfx checks
the configured root first and falls back to its standard system paths when a requested path does not
exist there. Treat `public/` as a complete tree: Websurfx selects one public directory and does not
fall back per missing asset inside it.

## Start the server

Set the data root before parsing configuration or starting the server. The root is process-global
and may only be set once, so a process can use one Websurfx data root:

```rust,no_run
use std::io;
use tokio::{net::TcpListener, sync::OnceCell};
use websurfx::{parser::Config, run, set_data_root};

static CONFIG: OnceCell<Config> = OnceCell::const_new();

#[tokio::main]
async fn main() -> io::Result<()> {
    set_data_root("websurfx-data")?;

    let config = CONFIG
        .get_or_try_init(|| async {
            Config::parse(false)
                .await
                .map_err(|error| io::Error::other(error.to_string()))
        })
        .await?;
    let listener = TcpListener::bind((config.binding_ip.as_str(), config.port)).await?;

    run(listener, config).await?.await
}
```

The host embeds Websurfx's Rust library code and calls its documented public API; implementation
modules remain private. The resource files remain ordinary runtime files managed by the host.
Relative data roots are resolved against the working directory at the time `set_data_root` is
called, so later working-directory changes do not redirect Websurfx to a different data directory.
