# Stable

To get started with Websurfx, clone the repository, edit the config file which is located in the `websurfx` directory and install redis server by following the instructions located [here](https://redis.io/docs/getting-started/) and then run the websurfx server and redis server using the following commands:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
cargo build
redis-server --port 8082 &
./target/debug/websurfx
```

# Rolling/Edge/Unstable

If you want to use the rolling/edge branch, run the following commands instead:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout rolling
cargo build
redis-server --port 8082 &
./target/debug/websurfx
```

Once you have started the server, open your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

# Docker Deployment

Before you start, you will need [Docker](https://docs.docker.com/get-docker/) installed on your system first.

## Stable

First clone the the repository by running the following command:

```bash
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
```

After that edit the config.lua file located under `websurfx` directory. In the config file you will specifically need to change to values which is `binding_ip_addr` and `redis_connection_url` which should make the config look something like this:

```lua
-- Server
port = "8080" -- port on which server should be launched
binding_ip_addr = "0.0.0.0" --ip address on the which server should be launched.

-- Website
-- The different colorschemes provided are:
-- {{
-- catppuccin-mocha
-- dracula
-- monokai
-- nord
-- oceanic-next
-- solarized-dark
-- solarized-light
-- tomorrow-night
-- }}
colorscheme = "catppuccin-mocha" -- the colorscheme name which should be used for the website theme
theme = "simple" -- the theme name which should be used for the website

-- Caching
redis_connection_url = "redis://redis:6379" -- redis connection url address on which the client should connect on.
```

After this run the following command to deploy the app:

```bash
docker compose up -d --build
```

This will take around 5-10 mins for first deployment, afterwards the docker build stages will be cached so it will be faster to be build from next time onwards. After the above step finishes launch your preferred browser and then navigate to `http://<ip_address_of_the_device>:<whatever_port_you_provided_in_the_config>`.

## Unstable/Edge/Rolling

For the unstable/rolling/edge version, follow the same steps as above with an addition of one command for cloning the repository which makes the cloning step as follows:

```bash
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout rolling
```

[⬅️  Go back to Home](https://github.com/neon-mmd/websurfx/wiki).
