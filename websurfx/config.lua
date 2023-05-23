-- Server
port = "8080" -- port on which server should be launched
binding_ip_addr = "127.0.0.1" --ip address on the which server should be launched.

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
redis_connection_url = "redis://127.0.0.1:8082" -- redis connection url address on which the client should connect on.

production_use = false -- whether to use production mode or not
-- if production_use is set to true
  -- there will be a random delay before sending the request to the search engines, this is to prevent the search engines from blocking the ip address