# Install From Package

## Arch Linux

You can install `Websurfx` through the [Aur](https://aur.archlinux.org/packages/websurfx-git), Currently we only support `Rolling/Edge` version. You can install the rolling/edge version by running the following command (using [paru](https://github.com/Morganamilo/paru)):

```bash
paru -S websurfx-edge-git
```

After installing it you can run the websurfx server by running the following commands:

```bash
redis-server --port 8082 &
websurfx
```

Once you have started the server, open your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

If you want to change the port or the ip or any other configuration setting checkout the [configuration docs](./configuration.md).

## Other Distros

The package is currently not available on other Linux distros. With contribution and support it can be made available on other distros as well 🙂.

# Install From Source

Before you can start building `websurfx`, you will need to have `Cargo` installed on your system. You can find the installation instructions [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Stable

To get started with Websurfx, clone the repository, edit the config file which is located in the `websurfx` directory and install redis server by following the instructions located [here](https://redis.io/docs/getting-started/) and then build and run the websurfx server by running the following commands:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout stable
cargo build -r
redis-server --port 8082 &
./target/release/websurfx
```

Once you have started the server, open your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

If you want to change the port or the ip or any other configuration setting checkout the [configuration docs](./configuration.md).

## Rolling/Edge/Unstable

If you want to use the rolling/edge branch, run the following commands instead:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
cargo build -r
redis-server --port 8082 &
./target/release/websurfx
```

Once you have started the server, open your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

If you want to change the port or the ip or any other configuration setting checkout the [configuration docs](./configuration.md).

# Docker Deployment

Before you start, you will need [Docker](https://docs.docker.com/get-docker/) installed on your system first.

## Unstable/Edge/Rolling

First clone the the repository by running the following command:

```bash
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
```

After that edit the config.lua file located under `websurfx` directory. In the config file you will specifically need to change to values which is `binding_ip_addr` and `redis_connection_url` which should make the config look something like this:

```lua
-- ### General ###
logging = true -- an option to enable or disable logs.
debug = false -- an option to enable or disable debug mode.

-- ### Server ###
port = "8080" -- port on which server should be launched
binding_ip_addr = "0.0.0.0" --ip address on the which server should be launched.
production_use = false -- whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users)
-- if production_use is set to true
-- There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests.

-- ### Website ###
-- The different colorschemes provided are:
-- {{
-- catppuccin-mocha
-- dark-chocolate
-- dracula
-- gruvbox-dark
-- monokai
-- nord
-- oceanic-next
-- one-dark
-- solarized-dark
-- solarized-light
-- tokyo-night
-- tomorrow-night
-- }}
colorscheme = "catppuccin-mocha" -- the colorscheme name which should be used for the website theme
theme = "simple" -- the theme name which should be used for the website

-- ### Caching ###
redis_connection_url = "redis://redis:6379" -- redis connection url address on which the client should connect on.

-- ### Search Engines ###
upstream_search_engines = { DuckDuckGo = true, Searx = false } -- select the upstream search engines from which the results should be fetched.
```

After this run the following command to deploy the app:

```bash
docker compose up -d --build
```

This will take around 5-10 mins for first deployment, afterwards the docker build stages will be cached so it will be faster to be build from next time onwards. After the above step finishes launch your preferred browser and then navigate to `http://<ip_address_of_the_device>:<whatever_port_you_provided_in_the_config>`.

## Stable

For the stable version, follow the same steps as above (as mentioned for the unstable/rolling/edge version) with an addition of one command which has to be performed after cloning and changing directory into the repository which makes the cloning step as follows:

```bash
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout stable
```

[⬅️ Go back to Home](./README.md)
