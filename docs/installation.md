# Install From Package

## Arch Linux

### Rolling/Edge/Unstable

You can install `Websurfx` through the [Aur](https://aur.archlinux.org/packages/websurfx-git), By running the following command (using [paru](https://github.com/Morganamilo/paru)):

```shell
paru -S websurfx-edge-git
```

After installing it you can run the websurfx server by running the following commands:

```shell
websurfx
```

Once you have started the server, open your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

If you want to change the port or the IP or any other configuration setting check out the [configuration docs](./configuration.md).

### Stable

For the stable version, follow the same steps as above (as mentioned for the `unstable/rolling/edge` version) with the only difference being that the package to be installed for the stable version is called `websurfx-git` instead of `websurfx-edge-git`.

## NixOS

A `flake.nix` has been provided to allow installing `websurfx` easily. It utilizes [nearsk](https://github.com/nix-community/naersk) to automatically generate a derivation based on `Cargo.toml` and `Cargo.lock`.

The Websurfx project provides 2 versions/flavours for the flake `stable` and `rolling/unstable/edge`. The steps for each are covered below in different sections.

### Rolling/Edge/Unstable

To get started, First, clone the repository, edit the config file which is located in the `websurfx` directory, and then build and run the websurfx server by running the following commands:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
cp -rf ./websurfx/ ~/.config/
$ mkdir /opt/websurfx/
$ cp -rf ./public/ /opt/websurfx/
nix build .#websurfx
nix run .#websurfx
```

> **Note**
> In the above command the dollar sign(**$**) refers to running the command in Privileged mode by using utilities `sudo`, `doas`, `pkgexec`, or any other privileged access methods.

Once you have run the above set of commands, open your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

If you want to change the port or the IP or any other configuration setting check out the [configuration docs](./configuration.md).

> Optionally, you may include it in your own flake by adding this repo to its inputs and adding it to `environment.systemPackages` as follows:
>
> ```nix
> {
>   description = "My awesome configuration";
>
>   inputs = {
>     websurfx.url = "github:neon-mmd/websurfx";
>   };
>
>   outputs = { nixpkgs, ... }@inputs: {
>     nixosConfigurations = {
>       hostname = nixpkgs.lib.nixosSystem {
>         system = "x86_64-linux";
>         modules = [{
>           environment.systemPackages = [inputs.websurfx.packages.x86_64-linux.websurfx];
>         }];
>       };
>     };
>   };
> }
> ```

### Stable

For the stable version, follow the same steps as above (as mentioned for the `unstable/rolling/edge version`) with an addition of one command which has to be performed after cloning and changing the directory into the repository which makes the building step as follows:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout stable
cp -rf ./websurfx/ ~/.config/
$ mkdir /opt/websurfx/
$ cp -rf ./public/ /opt/websurfx/
nix build .#websurfx
nix run .#websurfx
```

> **Note**
> In the above command the dollar sign(**$**) refers to running the command in privileged mode by using utilities `sudo`, `doas`, `pkgexec`, or any other privileged access methods.

## Other Distros

The package is currently not available on other Linux distros. With contribution and support it can be made available on other distros as well 🙂.

# Install From Source

Before you can start building `websurfx`, you will need to have `Cargo` installed on your system. You can find the installation instructions [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

## Stable

To get started with Websurfx, clone the repository, edit the config file which is located in the `websurfx` directory, and install redis server by following the instructions located [here](https://redis.io/docs/getting-started/) and then build and run the websurfx server by running the following commands:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout stable
cargo build -r
redis-server --port 8082 &
./target/release/websurfx
```

Once you have started the server, open your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

If you want to change the port or the IP or any other configuration setting check out the [configuration docs](./configuration.md).

## Rolling/Edge/Unstable

If you want to use the rolling/edge branch, run the following commands instead:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
```

Once you have changed the directory to the `websurfx` directory then follow the build options listed below:

### Hybrid Cache

> For more information on the features and their pros and cons. see: [**Features**](./features.md)

To build the search engine with the `Hybrid caching` feature. Run the following build command:

```shell
cargo build -r --features redis-cache
```

### Memory Cache (Default Features)

> For more information on the features and their pros and cons. see: [**Features**](./features.md)

To build the search engine with the `In-Memory caching` feature. Run the following build command:

```shell
cargo build -r
```

### No Cache

> For more information on the features and their pros and cons. see: [**Features**](./features.md)

To build the search engine with the `No caching` feature. Run the following build command:

```shell
cargo build -r --no-default-features
```

### Redis Cache

> For more information on the features and their pros and cons. see: [**Features**](./features.md)

To build the search engine with the `hybrid caching` feature. Run the following build command:

```shell
cargo build -r --no-default-features --features redis-cache
```

> Optionally, If you have built the app with the `Redis cache`or `Hybrid cache` feature (as mentioned above) then before launching the search engine run the following command:
>
> ```shell
> redis-server --port 8082 &
> ```

Once you have finished building the `search engine`. then run the following command to start the search engine:

```shell
./target/release/websurfx
```

Once you have started the server, launch your preferred web browser and navigate to http://127.0.0.1:8080/ to start using Websurfx.

If you want to change the port or the IP or any other configuration setting check out the [configuration docs](./configuration.md).

# Docker Deployment

Before you start, you will need [Docker](https://docs.docker.com/get-docker/) installed on your system first.

## Prebuild

The Websurfx project provides several prebuilt images based on the different features provided by the search engine. To get started using the prebuild image, you will first need to create a `docker-compose.yml` file with the following content:

```yaml
---
version: '3.9'
services:
  app:
    # Comment the line below if you don't want to use the `hybrid/latest` image.
    image: neonmmd/websurfx:latest
    # Uncomment the line below if you want to use the `no cache` image.
    # image: neonmmd/websurfx:nocache
    # Uncomment the line below if you want to use the `memory` image.
    # image: neonmmd/websurfx:memory
    # Uncomment the line below if you want to use the `redis` image.
    # image: neonmmd/websurfx:redis
    ports:
      - 8080:8080
    # Uncomment the following lines if you are using the `hybrid/latest` or `redis` image.
    # depends_on:
    #   - redis
    # links:
    #   - redis
    volumes:
      - ./websurfx/:/etc/xdg/websurfx/
  # Uncomment the following lines if you are using the `hybrid/latest` or `redis` image.
  # redis:
  #   image: redis:latest
```

Then make sure to edit the `docker-compose.yml` file as required. After that create a directory `websurfx` in the directory you have placed the `docker-compose.yml` file, and then in the new directory create two new empty files named `allowlist.txt` and `blocklist.txt`. Finally, create a new config file `config.lua` with the default configuration, which looks something like this:

```lua
-- ### General ###
logging = true -- an option to enable or disable logs.
debug = false -- an option to enable or disable debug mode.
threads = 8 -- the amount of threads that the app will use to run (the value should be greater than 0).

-- ### Server ###
port = "8080" -- port on which server should be launched
binding_ip = "0.0.0.0" --ip address on the which server should be launched.
production_use = false -- whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users (more than one))
-- if production_use is set to true
-- There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests.
request_timeout = 30 -- timeout for the search requests sent to the upstream search engines to be fetched (value in seconds).
rate_limiter = {
	number_of_requests = 20, -- The number of requests that are allowed within a provided time limit.
	time_limit = 3, -- The time limit in which the number of requests that should be accepted.
}

-- ### Search ###
-- Filter results based on different levels. The levels provided are:
-- {{
-- 0 - None
-- 1 - Low
-- 2 - Moderate
-- 3 - High
-- 4 - Aggressive
-- }}
safe_search = 2

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
colorscheme = "catppuccin-mocha" -- the colorscheme name that should be used for the website theme
theme = "simple" -- the theme name that should be used for the website

-- ### Caching ###
redis_url = "redis://redis:6379" -- redis connection url address on which the client should connect on.

-- ### Search Engines ###
upstream_search_engines = {
	DuckDuckGo = true,
	Searx = false,
} -- select the upstream search engines from which the results should be fetched.
```

Then run the following command to deploy the search engine:

```shell
$ docker compose up -d
```

> **Note**
> In the above command the dollar sign(**$**) refers to running the command in privileged mode by using utilities `sudo`, `doas`, `pkgexec` or any other privileged access methods.

Then launch the browser of your choice and navigate to http://<ip_address_of_the_device>:<whatever_port_you_provided_in_the_config>.

> **Note**
> The official prebuild images only support `stable` versions of the app and will not support `rolling/edge/unstable` versions. But with support and contribution, it could be made available for these versions as well 🙂.

## Manual Deployment

This section covers how to deploy the app with docker manually by manually building the image and deploying it.

> **Note**
> This section is provided for those who want to further customize the docker image or for those who are extra cautious about security.

> **Warning**
> A note of caution the project currently only supports **x86-64** architecture and as such we do not recommend deploying the project on devices with other architectures. Though if you still want to do it then **do it at your own risk**.

### Unstable/Edge/Rolling

First, clone the repository by running the following command:

```bash
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
```

After that edit the config.lua file located under `websurfx` directory. In the config file, you will specifically need to change to values which are `binding_ip_addr` and `redis_connection_url` which should make the config look something like this:

```lua
-- ### General ###
logging = true -- an option to enable or disable logs.
debug = false -- an option to enable or disable debug mode.
threads = 8 -- the amount of threads that the app will use to run (the value should be greater than 0).

-- ### Server ###
port = "8080" -- port on which server should be launched
binding_ip = "0.0.0.0" --ip address on the server should be launched.
production_use = false -- whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users (more than one))
-- if production_use is set to true
-- There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests.
request_timeout = 30 -- timeout for the search requests sent to the upstream search engines to be fetched (value in seconds).
rate_limiter = {
	number_of_requests = 20, -- The number of requests that are allowed within a provided time limit.
	time_limit = 3, -- The time limit in which the number of requests that should be accepted.
}

-- ### Search ###
-- Filter results based on different levels. The levels provided are:
-- {{
-- 0 - None
-- 1 - Low
-- 2 - Moderate
-- 3 - High
-- 4 - Aggressive
-- }}
safe_search = 2

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
redis_url = "redis://redis:6379" -- redis connection url address on which the client should connect on.

-- ### Search Engines ###
upstream_search_engines = {
	DuckDuckGo = true,
	Searx = false,
} -- select the upstream search engines from which the results should be fetched.
```

After this make sure to edit the `docker-compose.yml` and `Dockerfile` files as required and run the following command to deploy the app:

```bash
$ docker compose up -d --build
```

> **Note**
> In the above command the dollar sign(**$**) refers to running the command in privileged mode by using utilities `sudo`, `doas`, `pkgexec`, or any other privileged access methods.

This will take around 5-10 mins for the first deployment, afterwards, the docker build stages will be cached so it will be faster to build from next time onwards. After the above step finishes launch your preferred browser and then navigate to `http://<ip_address_of_the_device>:<whatever_port_you_provided_in_the_config>`.

### Stable

For the stable version, follow the same steps as above (as mentioned for the unstable/rolling/edge version) with an addition of one command which has to be performed after cloning and changing the directory into the repository which makes the cloning step as follows:

```bash
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout stable
```

[⬅️ Go back to Home](./README.md)
