# Build Options

The project provides 4 caching options as conditionally compiled features. This helps reduce the size of the compiled app by only including the code that is necessary for a particular caching option. 

The different caching features provided are as follows: 
- No cache
- Redis cache
- In memory cache
- Hybrid cache

## No Cache 

This feature disables caching for the search engine. This option can drastically reduce binary size but with the cost that subsequent search requests and previous & next page search results are not cached which can make navigating between pages slower. As well as page refreshes of the same page also becomes slower as each refresh has to fetch the results from the upstream search engines. 

To build the app with this option run the following command:

``` shell
cargo build -r --no-default-features
```

Once you have build the app with this option follow the commands listed on the [**Installation**](./installation.md#install-from-source) page of the docs to run the app.

## Redis Cache

This feature enables `Redis` caching ability for the search engine. This option allows the search engine to cache the results on the redis server. This feature can be useful for having a dedicated cache server for multiple devices hosted with the `Websurfx` server which can use the one dedicated cache server for hosting their cache on it. But a disadvantage of this solution is that if the `Redis`server is located far away (for example provided by a vps as service) and if it is unavailable or down for some reason then the `Websurfx` server would not be able to function properly or will crash on startup.

To build the app with this option run the following command:

``` shell
cargo build -r --no-default-features --features redis-cache
```

Once you have build the app with this option follow the commands listed on the [**Installation**](./installation.md#install-from-source) page of the docs to run the app.

## In Memory Cache 

This feature enables `In Memory` caching soluion within the search engine and it is the default feature provided by the project. This option allows the search engine to cache the results in the memory which can help increase the speed of the fetched cache results and it also has an advantage that it is extremely reliable as all the results are stored in memory within the search engine. Though the disadvantage of this solution are that caching of results is slightly slower than the `redis-cache` solution, it requires a good amount of memory on the system and as such is not ideal for very low memory devices and is highly unscalable.

To build the app with this option run the following command:

``` shell
cargo build -r
```

Once you have build the app with this option follow the commands listed on the [**Installation**](./installation.md#install-from-source) page of the docs to run the app.

## Hybrid Cache

This feature enables the `Hybrid` caching solution for the search engine which provides the advantages of both `In Memory` caching and `Redis` caching and it is an ideal solution if you need a very resiliant and reliable solution for the `Websurfx` which can provide both speed and reliability. Like for example if the `Redis` server becomes unavailable then the search engine switches to `In Memory` caching until the server becomes available again. This solution can be useful for hosting `Websurfx` instance which will be used by hundreds or thousands of users over the world.

To build the app with this option run the following command:

``` shell
cargo build -r --features redis-cache
```

Once you have build the app with this option follow the commands listed on the [**Installation**](./installation.md#install-from-source) page of the docs to run the app.

[⬅️ Go back to Home](./README.md)
