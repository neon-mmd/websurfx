# Features 

The project provides 4 caching options as conditionally compiled features. This helps reduce the size of the compiled app by only including the code that is necessary for a particular caching option. 

The different caching features provided are as follows: 
- No cache
- Redis cache
- In memory cache
- Hybrid cache

## Explanation

### No Cache 

This feature can drastically reduce binary size but with the cost that subsequent search requests and previous & next page search results are not cached which can make navigating between pages slower. As well as Page refreshes of the same page also become slower as each refresh has to fetch the results from the upstream search engines. 

### Redis Cache

This feature allows the search engine to cache the results on the redis server. This feature can be useful for having a dedicated cache server for multiple devices hosted with the `Websurfx` server which can use the one dedicated cache server for hosting their cache on it. But a disadvantage of this solution is that if the `Redis`server is located far away (for example provided by a vps as service) and if it is unavailable or down for some reason then the `Websurfx` server would not be able to function properly or will crash on startup.

### In Memory Cache 

This feature is the default feature provided by the project. This feature allows the search engine to cache the results in the memory which can help increase the speed of the fetched cache results and it also has the advantage that it is extremely reliable as all the results are stored in memory within the search engine. Though the disadvantage of this solution is that caching of results is slightly slower than the `redis-cache` solution, it requires a good amount of memory on the system and as such is not ideal for very low memory devices and is highly unscalable.

### Hybrid Cache

This feature provides the advantages of both `In Memory` caching and `Redis` caching and it is an ideal solution if you need a very resilient and reliable solution for the `Websurfx` which can provide both speed and reliability. Like for example if the `Redis` server becomes unavailable then the search engine switches to `In Memory` caching until the server becomes available again. This solution can be useful for hosting a `Websurfx` instance which will be used by hundreds or thousands of users all over the world.

## Tabular Summary 


| **Attributes**                          | **Hybrid** | **In-Memory**                                        | **No Cache**    | **Redis**              |
|-----------------------------------------|------------|------------------------------------------------------|-----------------|------------------------|
| **Speed**                               | Fast       | Caching is slow, but retrieval of cache data is fast | Slow            | Fastest                |
| **Reliability**                         | ✅          | ✅                                                    | ✅               | ❌                      |
| **Scalability**                         | ✅          | ❌                                                    | -               | ✅                      |
| **Resiliency**                          | ✅          | ✅                                                    | ✅               | ❌                      |
| **Production/Large Scale/Instance use** | ✅          | Not Recommended                                      | Not Recommended | Not Recommended        |
| **Low Memory Support**                  | ❌          | ❌                                                    | ✅               | ❌                      |
| **Binary Size**                         | Big        | Bigger than `No Cache`                               | small           | Bigger than `No Cache` |

[⬅️ Go back to Home](./README.md)
