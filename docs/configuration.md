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

# General

- **logging:** An option to enable or disable logs.
- **debug:** An option to enable or disable debug mode.

## Server

- **port:** Port number on which server should be launched.
- **binding_ip_addr:** IP address on the which server should be launched.
- **production_use:** Whether to use production mode or not (in other words this option should be used if it is to be used to host it on the server to provide a service to a large number of users). If production_use is set to true. There will be a random delay before sending the request to the search engines, this is to prevent DDoSing the upstream search engines from a large number of simultaneous requests. This is newly added option and hence is only available in the **edge version**.

## Website

- **colorscheme:** The colorscheme name which should be used for the website theme (the name should be in accordance to the colorscheme file name present in `public/static/colorschemes` folder).

> By Default we provide 9 colorschemes to choose from these are:
>
> 1. catppuccin-mocha
> 2. dracula
> 3. monokai
> 4. nord
> 5. oceanic-next
> 6. solarized-dark
> 7. solarized-light
> 8. tomorrow-night
> 9. gruvbox-dark

- **theme:** The theme name which should be used for the website (again, the name should be in accordance to the theme file name present in `public/static/themes` folder).

> By Default we provide 1 theme to choose from these are:
>
> 1. simple

## Cache

- **redis_connection_url:** Redis connection url address on which the client should connect on.

[⬅️  Go back to Home](./README.md)
