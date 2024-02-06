{
    server = {
	logging = true,
	debug = false,
	threads = 10,
	port = 8080,
	binding_ip = "127.0.0.1",
	aggregator = {
	    random_delay = false
	},
	request_timeout = 30,
	rate_limiter = {
	    number_of_requests = 20,
	    time_limit = 3
	}
    },

    caching = {
	-- redis_url = "redis://127.0.0.1::8082",
	cache_expiry_time = 600
    },

    search = {
	upstream_search_engines = {
	    DuckDuckGo = true,
       	    Searx = false,
       	    Brave = false,
       	    Startpage = false,
       	    LibreX = false,
       	    Mojeek = false,
       	    Bing = false,
    	},
	safe_search = 2
    },

    style = {
   	colorscheme = "catppuccin-mocha",
   	theme = "simple",
    	animation = "simple-frosted-glow"
    }
}
