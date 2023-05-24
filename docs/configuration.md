# Configuration

Everything in websurfx can be configured through the config file located at `websurfx/config.lua`.

Some of the configuration options provided in the file are stated below. These are subdivided into three categories:

- Server
- Website
- Cache

## Server

- **port:** Port number on which server should be launched.
- **binding_ip_addr:** IP address on the which server should be launched.

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

[⬅️  Go back to Home](https://github.com/neon-mmd/websurfx/wiki).
