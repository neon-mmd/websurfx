# Configuration

## Installed From Source

If you have built `websurfx` from the source then the configuration file will be located under the project directory (codebase) at `websurfx/`

> **Note**
> If you have built websurfx with an unstable/rolling/edge branch then you can copy the configuration file from `websurfx/config.lua` located under the project directory (codebase) to `~/.config/websurfx/` and make the changes there and rerun the websurfx server. _This is only available from unstable/rolling/edge version_.

## Installed From Package

If you have installed `websurfx` using the package manager of your Linux distro then the default configuration file will be located at `/etc/xdg/websurfx/`. You can copy the default config to `~/.config/websurfx/` make the changes there and rerun the websurfx server.

Some of the configuration options provided in the file are stated below. These are subdivided into the following categories:

- General
- Server
- Search
- Website
- Cache
- Search Engines

# General

- **logging:** An option to enable or disable logs.
- **debug:** An option to enable or disable debug mode.
- **threads:** The amount of threads that the app will use to run (the value should be greater than 0).

## Server

- **port:** Port number on which server should be launched.
- **binding_ip_addr:** IP address on the which server should be launched.
- **production_use:** Whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users). If production_use is set to true. There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests.
- **request_timeout:** Timeout for the search requests sent to the upstream search engines to be fetched (value in seconds).
- **rate_limiter:** The configuration option to configure rate limiting on the search engine website.

## Search

- **safe_search:** This option is used to configure the search filtering based on different safe search levels. (value a number between 0 to 4)

> This option provides 4 levels of search filtering:
>
> - Level 0 - With this level no search filtering occurs.
> - Level 1 - With this level some search filtering occurs.
> - Level 2 - With this level the upstream search engines are restricted to sending sensitive content like NSFW search results, etc.
> - Level 3 - With this level the regex-based filter lists are used alongside level 2 to filter more search results that have slipped in or custom results that need to be filtered using the filter lists.
> - Level 4 - This level is similar to level 3 except in this level the regex-based filter lists are used to disallow users to search sensitive or disallowed content. This level could be useful if you are a parent or someone who wants to completely disallow their kids or yourself from watching sensitive content.

## Website

- **colorscheme:** The colorscheme name which should be used for the website theme (the name should be by  the colorscheme file name present in the `public/static/colorschemes` folder).

> By Default we provide 12 colorschemes to choose from these are:
>
> 1. catppuccin-mocha
> 2. dark-chocolate
> 3. dracula
> 4. gruvbox-dark
> 5. monokai
> 6. nord
> 7. oceanic-next
> 8. one-dark
> 9. solarized-dark
> 10. solarized-light
> 11. tokyo-night
> 12. tomorrow-night

- **theme:** The theme name that should be used for the website (again, the name should be by the theme file name present in the `public/static/themes` folder).

> By Default we provide 1 theme to choose from these are:
>
> 1. simple

## Cache

- **redis_url:** Redis connection URL address on which the client should connect.

> **Note**
> This option can be commented out if you have compiled the app without the `redis-cache` feature. For more information, See [**building**](./building.md).

## Search Engines

- **upstream_search_engines:** Select from the different upstream search engines from which the results should be fetched.

[⬅️ Go back to Home](./README.md)
