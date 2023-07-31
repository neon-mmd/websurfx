# Configuration

## Installed From Source

If you have built `websurfx` from source then the configuration file will be located under project directory (codebase) at `websurfx/`

> **Note**
> If you have built websurfx with unstable/rolling/edge branch then you can copy the configuration file from `websurfx/config.lua` located under project directory (codebase) to `~/.config/websurfx/` and make the changes there and rerun the websurfx server. _This is only available from unstable/rolling/edge version_.

## Installed From Package

If you have installed `websurfx` using the package manager of your Linux distro then the default configuration file will be located at `/etc/xdg/websurfx/`. You can copy the default config to `~/.config/websurfx/` and make the changes there and rerun the websurfx server.

Some of the configuration options provided in the file are stated below. These are subdivided into the following categories:

- General
- Server
- Website
- Cache
- Search Engines

# General

- **logging:** An option to enable or disable logs.
- **debug:** An option to enable or disable debug mode.

## Server

- **port:** Port number on which server should be launched.
- **binding_ip_addr:** IP address on the which server should be launched.
- **production_use:** Whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users). If production_use is set to true. There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests. This is newly added option and hence is only available in the **edge version**.
- **request_timeout:** Timeout for the search requests sent to the upstream search engines to be fetched (value in seconds).

## Website

- **colorscheme:** The colorscheme name which should be used for the website theme (the name should be in accordance to the colorscheme file name present in `public/static/colorschemes` folder).

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

- **theme:** The theme name which should be used for the website (again, the name should be in accordance to the theme file name present in `public/static/themes` folder).

> By Default we provide 1 theme to choose from these are:
>
> 1. simple

## Cache

- **redis_url:** Redis connection url address on which the client should connect on.

## Search Engines

- **upstream_search_engines:** Select from the different upstream search engines from which the results should be fetched.

[⬅️ Go back to Home](./README.md)
