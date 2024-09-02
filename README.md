<h1 align="center">
  <img src="./images/websurfx_logo.png" alt="websurfx logo" align="center" />
</h1>
<p align="center">
  <b align="center"><a href="README.md">Readme</a></b> |
  <b><a href="https://discord.gg/SWnda7Mw5u">Discord</a></b> |
  <b><a href="docs/instances.md">Instances</a></b> |
  <b><a href="https://discord.gg/VKCAememnr">User Showcase</a></b> |
  <b><a href="https://github.com/neon-mmd/websurfx">GitHub</a></b> |
  <b><a href="docs">Documentation</a></b>
  <br /><br />
   <a
      href="https://github.com/awesome-selfhosted/awesome-selfhosted#search-engines"
   >
  <img
    src="https://cdn.rawgit.com/sindresorhus/awesome/d7305f38d29fed78fa85652e3a63e154dd8e8829/media/badge.svg"
    alt="Awesome Self-Hosted"
  />
  </a>
  <a href="#">
    <img
      alt="GitHub code size in bytes"
      src="https://img.shields.io/github/languages/code-size/neon-mmd/websurfx?style=flat-square"
    />
  </a>
  <a href="https://github.com/neon-mmd/websurfx/actions">
    <img
      alt="GitHub Workflow Status"
      src="https://img.shields.io/github/actions/workflow/status/neon-mmd/websurfx/rust.yml?style=flat-square"
    />
  </a>
  <a href=""
    ><img
      alt="Maintenance"
      src="https://img.shields.io/maintenance/yes/2024?style=flat-square"
    />
  </a>
  <a href="https://www.codefactor.io/repository/github/neon-mmd/websurfx">
    <img
      alt="CodeFactor"
      src="https://www.codefactor.io/repository/github/neon-mmd/websurfx/badge"
    />
  </a>
  <a href="https://gitpod.io/#https://github.com/neon-mmd/websurfx">
    <img
      alt="Gitpod"
      src="https://img.shields.io/badge/Gitpod-Ready--to--Code-blue?logo=gitpod"
    />
  </a>
  <br />
  <br />
  <i>
    A modern-looking, lightning-fast, privacy-respecting, secure
    <a href="https://en.wikipedia.org/wiki/Metasearch_engine"
      >meta search engine</a
    >
    (pronounced as websurface or web-surface /wɛbˈsɜːrfəs/.) written in Rust. It
    provides a quick and secure search experience while completely respecting user
    privacy.</i
  >
</p>

<details>
  <summary><b>Table of Contents</b></summary>
  <p>

- **Getting Started**
  - [🔭 Preview](#preview-)
  - [🚀 Features](#features-)
  - [🔗 Instances](#instances-)
  - [🛠️ Installation and Testing](#installation-and-testing-%EF%B8%8F)
  - [🔧 Configuration](#configuration-)
- **Feature Overview**
  - [🎨 Theming](#theming-)
  - [🌍 Multi-Language Support](#multi-language-support-)
- **Community**
  - [📊 System Requirements](#system-requirements-)
  - [🗨️ FAQ (Frequently Asked Questions)](#faq-frequently-asked-questions-%EF%B8%8F)
  - [📣 More Contributors Wanted](#more-contributors-wanted-)
  - [💖 Supporting Websurfx](#supporting-websurfx-)
  - [📘 Documentation](#documentation-)
  - [🛣️ Roadmap](#roadmap-%EF%B8%8F)
  - [🙋 Contributing](#contributing-)
  - [📜 License](#license-)
  - [🤝 Credits](#credits-)

  </p>
</details>

# Preview 🔭

## Home Page

<img align="center" src="./images/main_page.png" />

## Search Page

<img align="center" src="./images/search_page.png" />

## 404 Error Page

<img align="center" src="./images/404_error_page.png" />

**[⬆️ Back to Top](#--)**

# Instances 🔗

> For a full list of publicly available community driven `websurfx` instances to test or for daily use. see [**Instances**](docs/instances.md)

**[⬆️ Back to Top](#--)**

# Features 🚀

- 🎨 Make Websurfx uniquely yours with the twelve color schemes provided by default. It also supports the creation of custom themes and color schemes in a quick and easy way, so unleash your creativity!
- 🚀 Easy to setup with Docker or on bare metal with various installation and deployment options.
- ⛔ Search filtering to filter search results based on four different levels.
- 💾 Different caching levels focusing on reliability, speed and resiliancy.
- ⬆️  Organic Search results (with ranking algorithm builtin to rerank the search results according to user's search query.).
- 🔒 Different compression and encryption levels focusing on speed and privacy.
- 🧪 Experimental IO-uring feature for Linux operating systems focused on performance of the engine.
- 🔐 Fast, private, and secure
- 🆓 100% free and open source
- 💨 Ad-free and clean results
- 🌟 and lots more...

**[⬆️ Back to Top](#--)**

# Installation and Testing 🛠️

> For full setup instructions, see: [**Installation**](docs/installation.md)

Before you can start building `websurfx`, you will need to have `Cargo` installed on your system. You can find the installation instructions [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).

To get started with Websurfx, clone the repository, edit the config file, which is located in the `websurfx/` directory, and install the Redis server by following the instructions located [here](https://redis.io/docs/getting-started/) and then run the websurfx server and redis server using the following commands:

```shell
git clone https://github.com/neon-mmd/websurfx.git
cd websurfx
git checkout stable
cargo build -r
redis-server --port 8082 &
./target/release/websurfx
```

Once you have started the server, open your preferred web browser and navigate to <http://127.0.0.1:8080> to start using Websurfx.

> [!Note]
>
> 1. The project is no longer in the testing phase and is now ready for production use.
> 2. There are many features still missing, like `support for image search`, `different categories`, `quick apps`, etc., but they will be added soon as part of future releases.

**[⬆️ Back to Top](#--)**

# Configuration 🔧

> For full configuration instructions, see: [**Configuration**](docs/configuration.md)

Websurfx is configured through the config.lua file, located at `websurfx/config.lua`.

**[⬆️ Back to Top](#--)**

# Theming 🎨

> For full theming and customization instructions, see: [**Theming**](docs/theming.md)

Websurfx comes loaded with several themes and color schemes, which you can apply and edit through the config file. It also supports custom themes and color schemes using CSS, allowing you to make it truly yours.

**[⬆️ Back to Top](#--)**

# Multi-Language Support 🌍

> [!Note]
> Currently, we do not support other languages, but we will start accepting contributions regarding language support in the future. We believe language should never be a barrier to entry.

**[⬆️ Back to Top](#--)**

# System Requirements 📊

At present, we only support x86_64 architecture systems, but we would love to have contributions that extend to other architectures as well.

**[⬆️ Back to Top](#--)**

# FAQ (Frequently Asked Questions) 🗨️

## Why Websurfx?

The primary purpose of the Websurfx project is to create a fast, secure, and privacy-focused meta-search engine. There are numerous meta-search engines available, but not all guarantee the security of their search engines, which is critical for maintaining privacy. Memory flaws, for example, can expose private or sensitive information, which is understandably bad. There is also the added problem of spam, ads, and inorganic results, which most engines don't have a full-proof answer to. Until now. With Websurfx, I finally put a full stop to this problem. Websurfx is based on Rust, which ensures memory safety and removes such issues. Many meta-search engines also lack important features like advanced picture search, required by graphic designers, content providers, and others. Websurfx improves the user experience by providing these and other features, such as proper NSFW blocking and micro-apps or quick results (providing a calculator, currency exchanges, etc. in the search results).

## Why AGPLv3?

Websurfx is distributed under the **AGPLv3** license to keep the source code open and transparent. This helps keep malware, telemetry, and other dangers out of the project. **AGPLv3** is a strong copyleft license that ensures the software's source code, including any modifications or improvements made to the code, remains open and available to everyone.

## Why Rust?

Websurfx is based on Rust due to its memory safety features, which prevent vulnerabilities and make the codebase more secure. Rust is also faster than C++, contributing to Websurfx's speed and responsiveness. Finally, the Rust ownership and borrowing system enables secure concurrency and thread safety in the program.

**[⬆️ Back to Top](#--)**

# More Contributors Wanted 📣

We are looking for more willing contributors to help grow this project. For more information on how you can contribute, check out the [project board](https://github.com/neon-mmd/websurfx/projects?query=is%3Aopen) and the [CONTRIBUTING.md](CONTRIBUTING.md) file for guidelines and rules for making contributions.

**[⬆️ Back to Top](#--)**

# Supporting Websurfx 💖

> For full details and other ways you can help out, see: [**Contributing**](CONTRIBUTING.md)

If you use Websurfx and would like to contribute to its development, we're glad to have you on board! Contributions of any size or type are always welcome, and we will always acknowledge your efforts.

Several areas that we need a bit of help with at the moment are:

- **Better and more color schemes**: Help fix color schemes and add other famous color schemes.
- **Improve evasion code for bot detection**: Help improve code related to evading IP blocking and emulating human behaviors located in everyone's engine file.
- **Logo**: Help create a logo for the project and website.
- **Docker Support**: Help write a Docker Compose file for the project.
- Submit a PR to add a new feature, fix a bug, update the docs, add a theme, widget, or anything else.
- Star Websurfx on GitHub.

**[⬆️ Back to Top](#--)**

# Documentation 📘

> [!Note]
> We welcome any contributions to the [documentation](docs) as this will benefit everyone who uses this project.

**[⬆️ Back to Top](#--)**

# Roadmap 🛣️

> Coming soon! 🙂.

**[⬆️ Back to Top](#--)**

# Contributing 🙋

Contributions are welcome from anyone. It doesn't matter who you are; you can still contribute to the project in your own way.

## Not a developer but still want to contribute?

Check out this [video](https://youtu.be/FccdqCucVSI) by Mr. Nick on how to contribute.

## Developer

If you are a developer, have a look at the [CONTRIBUTING.md](CONTRIBUTING.md) document for more information.

**[⬆️ Back to Top](#--)**

# License 📜

Websurfx is licensed under the [AGPLv3](LICENSE) license.

**[⬆️ Back to Top](#--)**

# Credits 🤝

We would like to thank the following people for their contributions and support:

**Contributors**

<p>
  <br />
  <a href="https://github.com/neon-mmd/websurfx/graphs/contributors">
    <img src="https://contrib.rocks/image?repo=neon-mmd/websurfx" />
  </a>
  <br />
</p>

**Stargazers**

<p>
  <a href="https://github.com/neon-mmd/websurfx/stargazers">
    <img src="http://reporoster.com/stars/dark/neon-mmd/websurfx"/>
  </a>
</p>

**[⬆️ Back to Top](#--)**

---

<p align="center">
  <a href="https://github.com/neon-mmd/websurfx">
    <img src="https://github.githubassets.com/images/icons/emoji/octocat.png" />
  </a>
  <br /><br />
  <i>Thank you for Visiting</i>
</p>
