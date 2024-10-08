-- ### General ###
logging = true -- an option to enable or disable logs.
debug = false -- an option to enable or disable debug mode.
threads = 10 -- the amount of threads that the app will use to run (the value should be greater than 0).

-- ### Server ###
port = "8080" -- port on which server should be launched
binding_ip = "127.0.0.1" --ip address on the which server should be launched.
production_use = false -- whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users (more than one))
-- if production_use is set to true
-- There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests.
request_timeout = 30 -- timeout for the search requests sent to the upstream search engines to be fetched (value in seconds).
tcp_connection_keep_alive = 30 -- the amount of time the tcp connection should remain alive to the upstream search engines (or connected to the server). (value in seconds).
pool_idle_connection_timeout = 30 -- timeout for the idle connections in the reqwest HTTP connection pool (value in seconds).
rate_limiter = {
	number_of_requests = 20, -- The number of request that are allowed within a provided time limit.
	time_limit = 3, -- The time limit in which the quantity of requests that should be accepted.
}
-- Set whether the server will use an adaptive/dynamic HTTPS window size, see https://httpwg.org/specs/rfc9113.html#fc-principles
https_adaptive_window_size = false

number_of_https_connections = 10 -- the number of https connections that should be available in the connection pool.
-- Set keep-alive timer in seconds; keeps clients connected to the HTTP server, different from the connection to upstream search engines
client_connection_keep_alive = 120

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
-- The different themes provided are:
-- {{
-- simple
-- }}
theme = "simple" -- the theme name which should be used for the website
-- The different animations provided are:
-- {{
-- simple-frosted-glow
-- }}
animation = "simple-frosted-glow" -- the animation name which should be used with the theme or `nil` if you don't want any animations.

-- ### Caching ###
redis_url = "redis://127.0.0.1:8082" -- redis connection url address on which the client should connect on.
cache_expiry_time = 600 -- This option takes the expiry time of the search results (value in seconds and the value should be greater than or equal to 60 seconds).
-- ### Search Engines ###
upstream_search_engines = {
    DuckDuckGo = true,
    Searx = false,
    Brave = false,
    Startpage = false,
    LibreX = false,
    Mojeek = false,
    Bing = false,
} -- select the upstream search engines from which the results should be fetched.

proxy = nil -- Proxy to send outgoing requests through. Set to nil to disable.