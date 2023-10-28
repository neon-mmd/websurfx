# Developing

This page of the docs outlines how to get **Websurfx** up and running in a development environment, and outlines the common workflow, different ways to work on the project, a high-level overview of how the project works, project structure, and the best practices that should be followed when working on the project.

<details>
  <summary><b>Table of Contents</b></summary>
  <p>

- [Setting up the Development Environment](#setting-up-the-development-environment)
  - [Local Development](#local-development-)
  - [Gitpod](#gitpod-)
  - [NixOS Dev Shell using Nix Flake](#nixos-dev-shell-using-nix-flake-)
  - [Local Development with Docker Compose](#local-development-with-docker-compose-)
  - [Project Commands](#project-commands)
+ - [Environment Variables](#environment-variables)
- [Git Strategy](#git-strategy)
  - [Flow](#git-flow)
  - [Branches](#git-branch-naming)
  - [Commit emojis](#commit-emojis)
  - [PR Guidelines](#pr-guidelines)
- [Resources for Beginners](#resources-for-beginners)
- [App Info](#app-info)
- [Code Style Guide](#style-guide)
- [Application Structure](#application-structure)
- [Development Tools](#development-tools)
- [Misc / Notes](#notes)

  </p>
</details>

## Setting up the Development Environment

By default, we provide four different ways to work on the project. These are as follows:

- [Local Development](#local-development-)
- [Gitpod](#gitpod-)
- [NixOS Dev Shell using Nix Flake](#nixos-dev-shell-using-nix-flake-)
- [Local Development with Docker Compose](#local-development-with-docker-compose-)

The different methods are explained in depth below.

### Local Development

This section covers how to set up the project for development on your local machine (bare metal).

#### Prerequisites

Before you start working on the project. You will need the following packages installed on your system:

- The latest version of `cargo` installed on your system which is required to manage building and running the project. The installation instructions for this can be found [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).
- The latest version of `npm` installed on your system which is required to allow the installation of other tools necessary for the project. The installation for this can be found [here](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm).
- The latest version of `redis` installed on your system which will be used to avoid introducing unexpected issues when working on the project. The installation for this can be found [here](https://redis.io/docs/getting-started/installation/).
- The latest version of `stylelint` should be installed on your system which will be used by the pre-commit checks to lint the code before a commit can be made to ensure better code quality. Before you install `stylelint` on your system, make sure you have `npm` installed on your system. To install `stylelint` run the following command:

```shell
$ npm i -g stylelint
```

> **Note**
> In the above command the dollar sign(**$**) refers to running the command in privileged mode by using utilities `sudo`, `doas`, `pkgexec`, or any other privileged access methods.

- `Cargo-watch` installed on your system which will allow you to auto-build the project when any checks occur in the source code files in the codebase (`websurfx` directory). Before you install `cargo-watch` on your system, make sure you have `cargo` installed on your system. To install `cargo-watch` run the following command:

```shell
cargo install cargo-watch
```

- `Git` installed on your system. The installation instructions for this can be found [here](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git).
- Finally, The latest version of `Docker` is installed on your system which will be used to avoid introducing unexpected issues when working on the project. The installation instructions for this can be found [here](https://docs.docker.com/engine/install/).

> **Note**
> For **rolling release Linux distributions (distros)**, the above-mentioned required packages except for `stylelint` and `cargo-watch` can also be installed via the distro-specific package manager.
>
> **For Example:**
>
> On `arch linux` the following packages can be installed by following the link to the installation instructions provided below:
>
> - `Cargo`: https://wiki.archlinux.org/title/rust
> - `Npm`: https://wiki.archlinux.org/title/Node.js
> - `Redis`: https://wiki.archlinux.org/title/redis
> - `Git`: https://wiki.archlinux.org/title/git
> - `Docker`: https://wiki.archlinux.org/title/docker
>
> But we do not recommend this method for **stable release Linux distros** as they tend to not provide very up-to-date versions of the required packages.

#### Setting up Pre-commit Checks

Before you set `pre-commit` checks, you will first need to clone **your fork of the project** and navigate into the cloned repository by running the following command:

```shell
git clone https://github.com/<your_github_username>/websurfx.git
cd websurfx
```

Once you have finished running the above commands then run the following command to set the `pre-commit` checks:

```shell
cargo test
```

By running the above-mentioned command, it will automatically set up all the pre-commit checks in the project.

#### Running the Project

If you have followed the above section then you should have a cloned repository folder present on your system. In the same directory run the following command to run the project:

```shell
cargo watch -q -x "run" -w "."
```

This will compile the app by default with the **In-Memory caching** feature. To compile, run, and test the app with other features follow the build options listed below:

##### Hybrid Cache

To build and run the app with the `Hybrid caching` feature. Run the following command:

```shell
cargo watch -q -x "run --features redis-cache" -w .
```

##### No Cache

To build and run the search engine with the `No caching` feature. Run the following command:

```shell
cargo watch -q -x "run --no-default-features" -w .
```

##### Redis Cache

To build the search engine with the `Redis caching` feature. Run the following command:

```shell
cargo watch -q -x "run --no-default-features --features redis-cache" -w .
```

> Optionally, If you have build and run the app with the `Redis cache`or `Hybrid cache` feature (as mentioned above) then you will need to start the redis server alongside the app which can be done so by running the following command:
>
> ```shell
> redis-server --port 8082 &
> ```

Once you have finished running the above command, Websurfx should now be served on the address http://127.0.0.1:8080. Hot reload is enabled, so making changes to any of the files will trigger the project to be rebuilt.

> For more info on all the project commands. See: [**Project Commands**](#project-commands-)

### Gitpod

This section covers how to use and set up the Gitpod development environment for working on the project.

> **Note**
> By default the project only supports the Vscode **IDE/Editor** for Gitpod.

#### Launching Gitpod

> For a full guide on how to fork the project. See: [**Forking**](#)

To launch gitpod and start working on the project from your fork of the Websurfx, Just navigate to the following link:

```text
https://gitpod.io/#https://github.com/<your_github_username>/websurfx
```

> For a full guide on how to use it and how to use it in different ways. See [**Learn Gitpod**](https://piped.kavin.rocks/playlist?list=PL3TSF5whlprXVp-7Br2oKwQgU4bji1S7H)

#### Default Plugins

The project by default provides a set of pre-installed plugins for gitpod which is done to improve productivity and efficiency while working on the project. Also to make working on the project more fun and engaging which can be customized from within the `Gitpod` instance.

The list of all the pre-installed plugins are listed below:

**Productivity**

- [CodeLLDB](https://open-vsx.org/extension/vadimcn/vscode-lldb): Provides a native debugger for rust programming langauge.
- [GitHub Actions](https://open-vsx.org/extension/cschleiden/vscode-github-actions): Provides an easy to work with github actions.
- [rust-analyzer](https://open-vsx.org/extension/rust-lang/rust-analyzer): Provides a language server for rust programming langauge.
- [better-toml](https://open-vsx.org/extension/bungcip/better-toml): Provides support for toml files.
- [crates](https://open-vsx.org/extension/serayuzgur/crates): Makes managing rust dependencies easier.
- [Error Lens](https://open-vsx.org/extension/usernamehw/errorlens): Provides better highlighting of errors.
- [markdownlint](https://open-vsx.org/extension/DavidAnson/vscode-markdownlint): Provides a linter for linting markdown documents.
- [Prettier](https://open-vsx.org/extension/esbenp/prettier-vscode): Provides a code formatter.
- [Stylelint](https://open-vsx.org/extension/stylelint/vscode-stylelint): Provides a linter for CSS files.
- [ESLint](https://open-vsx.org/extension/dbaeumer/vscode-eslint): Provides a linter for JS files.
- [Syntax Highlighter](https://open-vsx.org/extension/evgeniypeshkov/syntax-highlighter): A better syntax highlighting for code.
- [Docker](https://open-vsx.org/extension/ms-azuretools/vscode-docker): Makes handling docker files easier.
- [indent-rainbow](https://open-vsx.org/extension/oderwat/indent-rainbow): Highlightes code idents for better visualization.
- [Auto Rename Tag](https://open-vsx.org/extension/formulahendry/auto-rename-tag): Provides a way to easily and quickly rename html tags.
- [Rust Test Explorer](https://open-vsx.org/extension/Swellaby/vscode-rust-test-adapter): View and run cargo tests easily from a convenient sidebar.
- [Search crates-io](https://open-vsx.org/extension/belfz/search-crates-io): Provides crates suggestions in the `cargo.toml` file.
- [Test Adapter Converter](https://open-vsx.org/extension/hbenl/test-adapter-converter): A vscode native way to view and run tests.
- [Test Explorer UI](https://open-vsx.org/extension/hbenl/vscode-test-explorer): Provides a way to run any test from a convenient sidebar.
- [GitLens](https://open-vsx.org/extension/eamodio/gitlens): Provides a better and more efficient way to manage common git workflows.

> Optionally, if you prefer a more keyboard-centric workflow then we would recommend using the following extension:
>
> - [VSCode Neovim](https://open-vsx.org/extension/asvetliakov/vscode-neovim): Provides complete vim emulation for vscode.

**Theming**

- [Catppuccin for VSCode](https://open-vsx.org/extension/Catppuccin/catppuccin-vsc): Provides the catpuccin theme for vscode.
- [Material Icon Theme](https://open-vsx.org/extension/PKief/material-icon-theme): Provides material design icons for files dependening on the file extension.

> If you have more ideas and ways to improve Gitpod for development purposes then feel free to do so by contributing a PR to this project [**here**](https://github.com/neon-mmd/websurfx/pulls).

### NixOS Dev Shell using Nix Flake

This section covers how to setup the project for development using the `NixOS dev-shell`.

#### Pre Setup Requirements

Before you start working on the project. You will need the following packages installed on your system:

- `Git` installed on your system. The installation instructions for this can be found [here](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git).
- Finally, The latest version of `Docker` is installed on your system which will be used to avoid introducing unexpected issues when working on the project. The installation instructions for this can be found [here](https://docs.docker.com/engine/install/).

> Optionally, On `NixOS` the above-mentioned required packages except for `stylelint` and `cargo-watch` could also be installed by following the link to the installation instructions provided below:
>
> - `Git`: https://search.nixos.org/packages?channel=23.05&show=git&from=0&size=50&sort=relevance&type=packages&query=git
> - `Docker`: https://search.nixos.org/packages?channel=23.05&show=docker&from=0&size=50&sort=relevance&type=packages&query=docker

#### Setting up Pre-commit Checks

Before you setup `pre-commit` checks, you will first need to clone **your fork of the project** and navigate into the cloned repository by running the following command:

```shell
git clone https://github.com/<your_github_username>/websurfx.git
cd websurfx
```

Then run the following command to setup the `NixOS dev-shell`:

```shell
nix develop
```

Once you have finished running the above commands then run the following command to setup the `pre-commit` checks:

```shell
cargo test
```

By running the above-mentioned command, it will automatically set up all the pre-commit checks in the project.

#### Post Setup Requirements

After you have done setting up pre-commit checks, then you may need to fulfill a few more requirements to finish setting up the development environment with `NixOS dev-shell`. These include:

- `Cargo-watch` installed on your system which will allow you to auto-build the project when any checks occur in the source code files in the codebase (`websurfx` directory). Before you install `cargo-watch` on your system, make sure you have `cargo` installed on your system. To install `cargo-watch` run the following command:

```shell
cargo install cargo-watch
```

#### Running the Project

If you have followed the above section then you should now be inside a `dev-shell` environment. In the same environment run the following command to run the project:

```shell
cargo watch -q -x "run" -w "."
```

This will compile the app by default with the **In-Memory caching** feature. To compile, run, and test the app with other features follow the build options listed below:

##### Hybrid Cache

To build and run the app with the `Hybrid caching` feature. Run the following command:

```shell
cargo watch -q -x "run --features redis-cache" -w .
```

##### No Cache

To build and run the search engine with the `No caching` feature. Run the following command:

```shell
cargo watch -q -x "run --no-default-features" -w .
```

##### Redis Cache

To build the search engine with the `Redis caching` feature. Run the following command:

```shell
cargo watch -q -x "run --no-default-features --features redis-cache" -w .
```

> Optionally, If you have build and run the app with the `Redis cache`or `Hybrid cache` feature (as mentioned above) then you will need to start the redis server alongside the app which can be done by running the following command:
>
> ```shell
> redis-server --port 8082 &
> ```

Once you have finished running the above command, Websurfx should now be served on the address http://127.0.0.1:8080. Hot reload is enabled, so making changes to any of the files will trigger the project to be rebuilt.

### Local Development with Docker Compose

This section covers how to set up the project for development on your local machine (bare metal) using `docker compose`.

#### Prerequisites

Before you start working on the project. You will need the following packages installed on your system:

- The latest version of `cargo` installed on your system which is required to manage the building and running the project. The installation instructions for this can be found [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).
- The latest version of `npm` installed on your system which is required to allow the installation of other tools necessary for the project. The installation for this can be found [here](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm).
- The latest version of `stylelint` should be installed on your system which will be used by the pre-commit checks to lint the code before a commit can be made to ensure better code quality. Before you install `stylelint` on your system, make sure you have `npm` installed on your system. To install `stylelint` run the following command:

```shell
$ npm i -g stylelint
```

> **Note**
> In the above command the dollar sign(**$**) refers to running the command in privileged mode by using utilities `sudo`, `doas`, `pkgexec`, or any other privileged access methods.

- `Git` installed on your system. The installation instructions for this can be found [here](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git).
- Finally, The latest version of `Docker` is installed on your system which will be used to avoid introducing unexpected issues when working on the project. The installation instructions for this can be found [here](https://docs.docker.com/engine/install/).

> **Note**
> For **rolling release Linux distributions (distros)**, the above-mentioned all required packages can also be installed via the distro-specific package manager.
>
> **For Example:**
>
> On `arch linux` the following packages can be installed by following the link to the installation instructions provided below:
>
> - `Cargo`: https://wiki.archlinux.org/title/rust
> - `Npm`: https://wiki.archlinux.org/title/Node.js
> - `Git`: https://wiki.archlinux.org/title/git
> - `Docker`: https://wiki.archlinux.org/title/docker
>
> But we do not recommend this method for **stable release Linux distros** as they tend to not provide very up-to-date versions of the required packages.

#### Setting up Pre-commit Checks

Before you setup `pre-commit` checks, you will first need to clone **your fork of the project** and navigate into the cloned repository by running the following command:

```shell
git clone https://github.com/<your_github_username>/websurfx.git
cd websurfx
```

Once you have finished running the above commands then run the following command to setup the `pre-commit` checks:

```shell
cargo test
```

By running the above-mentioned command, it will automatically set up all the pre-commit checks in the project.

#### Running the Project

If you have followed the above section then you should have a cloned repository folder present on your system. In the same directory, edit the `dev.docker-compose.yml` file as required before running the following command to run the project:

```shell
$ docker compose -f dev.docker-compose.yml up
```

> **Note**
> In the above command the dollar sign(**$**) refers to running the command in privileged mode by using utilities `sudo`, `doas`, `pkgexec`, or any other privileged access methods.

Once you have finished running the above command, Websurfx should now be served on the address http://127.0.0.1:8080. Hot reload is enabled, so making changes to any of the files will trigger the project to be rebuilt.

### Project Commands

#### Basics

- `cargo build`: Builds the project.

> **Note**
> When you build the project first time with the above command it will require the app to compile every dependency in the project which will then be cached on your system. So when you compile the app next time it will only compile for the new changes.

+ `cargo run`: Starts the app and serves the project on http://127.0.0.1:8080.


> **Important**
> You must run the build command first.

#### Development

- `cargo watch -q -x "run" -w .`: Starts the development server with hot reloading.
- `cargo fmt -- --check`: Checks the code for proper formatting.
- `cargo clippy`: Lints code to ensure it follows a consistent, neat style.
- `cargo test`: Runs unit tests, integrations tests and doc tests.

### Environment Variables

All environment variables are optional. Currently, there are not many environment variables used, as most of the user preferences are stored under the `websurfx` folder (located under the codebase (`websurfx` directory)) in the `config.lua` file.

The list of all the available environment variables are listed below:

- `PKG_ENV`: Sets the logging level for the app to **Trace** which can be useful for better debugging of the app. These environment variables accept two values `dev` or `prod` as strings.
- `RUST_BACKTRACE`: Rust-specific environment variable useful for getting more elaborate error messages with an error stack to better diagnose the issue. This environment variable accepts three values `0` (off), `1` (on), and `full` (for long error stack to being printed out).

## Git Strategy

### Git Flow

Like most Git repos, we are following the [Github Flow](https://guides.github.com/introduction/flow) standard.

1. Create a branch (or fork if you don't have write access)
2. Code some awesome stuff 🧑‍💻
3. Add, commit, and push your changes to your branch/ fork
4. Head over to GitHub and create a Pull Request
5. Fill in the required sections in the template, and hit submit
6. Follow up with any reviews on your code
7. Merge 🎉

### Git Branch Naming

The format of your branch name should be something similar to: `[TYPE]/[TICKET]_[TITLE]`
For example, `FEATURE/420_Awesome-feature` or `FIX/690_login-server-error`

### Commit Emojis

Using a single emoji at the start of each commit message, issue title, and pull request title, to indicate the type of task, makes the commit ledger, issue, and pull request easier to understand, it looks cool.

- 🎨 `:art:` - Improve the structure/format of the code.
- ⚡️ `:zap:` - Improve performance.
- 🔥 `:fire:` - Remove code or files.
- 🐛 `:bug:` - Fix a bug.
- 🚑️ `:ambulance:` - Critical hotfix
- ✨ `:sparkles:` - Introduce new features.
- 📝 `:memo:` - Add or update documentation.
- 🚀 `:rocket:` - Deploy stuff.
- 💄 `:lipstick:` - Add or update the UI and style files.
- 🎉 `:tada:` - Begin a project.
- ✅ `:white_check_mark:` - Add, update, or pass tests.
- 🔒️ `:lock:` - Fix security issues.
- 🔖 `:bookmark:` - Make a Release or Version tag.
- 🚨 `:rotating_light:` - Fix compiler/linter warnings.
- 🚧 `:construction:` - Work in progress.
- ⬆️ `:arrow_up:` - Upgrade dependencies.
- 👷 `:construction_worker:` - Add or update the CI build system.
- ♻️ `:recycle:` - Refactor code.
- 🩹 `:adhesive_bandage:` - Simple fix for a non-critical issue.
- 🔧 `:wrench:` - Add or update configuration files.
- 🍱 `:bento:` - Add or update assets.
- 🗃️ `:card_file_box:` - Perform database schema-related changes.
- ✏️ `:pencil2:` - Fix typos.
- 🌐 `:globe_with_meridians:` - Internationalization and translations.

For a full list of options, see [gitmoji.dev](https://gitmoji.dev/)

### PR Guidelines

Once you've made your changes, and pushed them to your fork or branch, you're ready to open a pull request!

For a pull request to be merged, it must:

- The build, lint, and tests (run by GH actions) must pass
- There must not be any merge conflicts

When you submit your pull request, include the required info, by filling out the pull request template. Including:

- A brief description of your changes.
- The issue or ticket number (if applicable).
- For UI-related updates include a screenshot.
- If any dependencies were added, explain why it was needed, and state the cost. associated, and confirm it does not introduce any security, privacy, or speed issues
- Optionally, provide a checklist of all the changes that were included in the pull request.

> **Important**
> Make sure to fill all the required/mandatory sections of the pull request as filling them helps us distinguish between spam pull requests and legitimate pull requests.

> **Note**
> The pull request template contains comments in the following form `<!-- -->` which are used to provide a guide on what should be provided under each heading of the template. These comments are never rendered when the pull request is either created or updated and hence anything provided in such comments is never displayed.

## Resources for Beginners

New to Web Development? Or New to GitHub? Glad to see you're here!! :slightly_smiling_face: Websurfx is a pretty simple app, so it should make a good candidate for your first PR. The following articles (which have been divided into parts for convenience) should point you in the right direction for getting up to speed with the technologies used in this project:

**Development**

- [Basics of Rust](https://piped.kavin.rocks/playlist?list=PLai5B987bZ9CoVR-QEIN9foz4QCJ0H2Y8)
- [Introduction and deep dive into async/await in rust](https://piped.kavin.rocks/watch?v=ThjvMReOXYM)
- [Getting Started to Actix Guide](https://actix.rs/docs/getting-started)
- [Basics of Lua](https://learn.coregames.com/courses/intro-to-lua/)
- [Complete course on CSS](https://piped.kavin.rocks/watch?v=1Rs2ND1ryYc)
- [Complete course on JS](https://piped.kavin.rocks/playlist?list=PL_c9BZzLwBRLVh9OdCBYFEql6esA6aRsi)
- [Responsive web design](https://piped.kavin.rocks/watch?v=srvUrASNj0s)
- [Complete beginners guide to Docker](https://docker-curriculum.com/)
- [Docker Classroom - Interactive Tutorials](https://training.play-with-docker.com/)
- [Docker Compose Tutorial](https://docs.docker.com/compose/gettingstarted/)
- [ES6 Tutorial](https://piped.kavin.rocks/watch?v=nZ1DMMsyVyI)
- [Cargo Guide Book](https://doc.rust-lang.org/cargo/index.html)

**GitHub**

- [Complete Guide to Open Source - How to Contribute](https://piped.kavin.rocks/watch?v=yzeVMecydCE)
- [Forking a Project](https://piped.kavin.rocks/watch?v=FnxFwyzm4Z4)
- [A Tutorial on Git](https://piped.kavin.rocks/playlist?list=PL4lTrYcDuAfxAgSefXftJXbhw0qvjfOFo)
- [Git cheat sheet](http://git-cheatsheet.com/)

For Rust, CSS, JS, HTML, Git, and Docker- you'll need an IDE (e.g. [VSCode](https://code.visualstudio.com/) or [Neovim](https://neovim.io/) and a terminal (Windows users may find [WSL](https://docs.microsoft.com/en-us/windows/wsl/) more convenient).

## App Info

### Style Guides

Linting is done using [Cargo Clippy](https://doc.rust-lang.org/clippy/) and [StyleLint](https://stylelint.io/) or [ESLint](https://eslint.org/). Also, linting is run as a git pre-commit hook.

> **Important**
> All lint checks must pass before any PR can be merged.

Styleguides to follow:

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/naming.html)
- [Airbnb JS Guidelines](https://github.com/airbnb/javascript)
- [Google's Html and CSS Guidelines](https://google.github.io/styleguide/htmlcssguide.html)

## Application Structure

> **Important**
> We follow the Unix style naming conventions for all the files and folders in the project (except for all files under the `themes` and `colorschemes` folder in the frontend's source code which requires that the names of the files and folders should be in lowercase and the words be separated with a hyphen.) which includes the name of the files and folders should be in lowercase and every word should be separated with an underscore.

**Files in the root of the codebase:** `./`

```
./
├── .dockerignore                # Docker ignore file to ignore stuff being included in the file docker image.
├── .gitignore                   # Git ignore file to ignore stuff from being
├── Cargo.lock                   # Auto-generated list of current packages and version numbers.
├── Cargo.toml                   # Project meta-data and dependencies.
├── Dockerfile                   # The blueprint for building the Docker container.
├── LICENSE                      # License for use.
├── README.md                    # Readme, basic info for getting started.
├── dev.Dockerfile               # The blueprint for building the Docker container for development purposes.
├── dev.docker-compose.yml       # A Docker run command for development environments.
├── docker-compose.yml           # A Docker run command.
├── flake.lock                   # NixOS auto-generated flake configuration.
├── flake.nix                    # Nix flake package configuration.
├── docs                         # Markdown documentation
├── public                       # Project front-end source code
├── src                          # Project back-end source code
├── tests                        # Project integration tests for the back-end source code.
└── websurfx                     # Project folder containing config files for the app.
```

**Frontend Source:** `./public/`

```
./public/
├── robots.txt                    # Robots file for the Website.
├── images                        # Images for the Website.
├── static                        # The directory containing all the UI handlers.
│   ├── cookies.js                # Handles the loading of saved cookies.
│   ├── error_box.js              # Handles the toggling functionality of the error box on the search page.
│   ├── index.js                  # Functions to handle the search functionality of the search bar.
│   ├── pagination.js             # Functions to handle the navigation between the previous and next page in the search page.
│   ├── search_area_options.js    # Changes the search options under the search bar in the search page according to the safe search level set using the URL safesearch parameter.
│   ├── settings.js               # Handles the settings and saving of all the settings page options as a cookie.
│   ├── colorschemes              # A folder containing all the popular colorscheme files as CSS files.
│   └── themes                    # A folder containing all the popular theme files as CSS files.
└── templates                     # Folder containing all the template files for the different pages on the website.
    ├── 404.html                  # A 404-page template.
    ├── about.html                # An about page template.
    ├── bar.html                  # A template for the search bar.
    ├── cookies_tab.html          # A template for the cookies tab for the settings page.
    ├── engines_tab.html          # A template for the engines tab for the settings page.
    ├── footer.html               # A footer template for all pages.
    ├── general_tab.html          # A template for the general tab for the settings page.
    ├── header.html               # A header template for all pages.
    ├── index.html                # A home page template.
    ├── navbar.html               # A navbar template for the header template.
    ├── search.html               # A search page template.
    ├── search_bar.html           # A search bar template specifically for the search page.
    ├── settings.html             # A settings page template.
    └── user_interface_tab.html   # A template for the user interface tab for the settings page.
```

**Backend Source:** `./src/`

```
./src/
├── lib.rs                        # A library file for the rust project.
├── bin                           # A folder containing the source code that would produce the binary file when compiled.
│   └── websurfx.rs               # A file that would be compiled into a binary file.
├── cache                         # A folder that contains code to handle the caching functionality of the search engine.
│   ├── cacher.rs                 # Handles the different caching features.
│   ├── error.rs                  # Provides custom error messages for different types of caches and their related errors.
│   ├── mod.rs                    # A module file for the rust project.
│   └── redis_cacher.rs           # Provides custom asynchronous pool implementation with auto background reconnection functionality.
├── config                        # A folder that holds the code to help parse the lua config file that would be used in the app.
│   ├── mod.rs                    # A module file for the rust project.
│   └── parser.rs                 # Provides the code to parse the config file.
├── engines                       # A folder that holds code to handle fetching data from different upstream engines.
│   ├── brave.rs                  # Provides code to fetch and remove unnecessary or waste results from the fetched results from the brave search engine.
│   ├── duckduckgo.rs             # Provides code to fetch and remove unnecessary or waste results from the fetched results from the duckduckgo search engine.
│   ├── mod.rs                    # A module file for the rust project.
│   ├── search_result_parser.rs   # Provides helper function to help ease the process of defining different result selection selectors.
│   └── searx.rs                  # Provides code to fetch and remove unnecessary or waste results from the fetched results from the searx engine.
├── handler                       # A folder that provides helper code to provide a proper path to the public (theme) folder, config file, blocklist file, and allowlist file based on where they are located.
│   ├── mod.rs                    # A module file for the rust project.
│   └── paths.rs                  # Provides helper code to handle different paths.
├── models                        # A folder that provides different models for the different modules in the backend code.
│   ├── aggregation_models.rs     # Provides different models (enums, structs) for handling and standardizing different parts in the "results" module code.
│   ├── engine_models.rs          # Provides different models (enums, structs) for handling and standardizing different parts in the "engines" module code.
│   ├── mod.rs                    # A module file for the rust project.
│   ├── parser_models.rs          # Provides different models (enums, structs) for handling and standardizing different parts in the "config" module code.
│   └── server_models.rs          # Provides different models (enums, structs) for handling and standardizing different parts in the "server" module code.
├── results                       # A folder that provides code to handle the fetching and aggregating of results from the upstream search engines.
│   ├── aggregator.rs             # Provides code aggregate and fetches results from the upstream engines.
│   ├── mod.rs                    # A module file for the rust project.
│   └── user_agent.rs             # Provides a helper function to allow random user agents to pass in the server request code to improve user privacy and avoiding detected as a bot.
└── server                        # A folder that holds code to handle the routes for the search engine website.
    ├── mod.rs                    # A module file for the rust project.
    ├── router.rs                 # Provides functions to handle the different routes on the website.
    └── routes                    # A folder that contains code to handle the bigger route for the website.
        ├── mod.rs                # A module file for the rust project.
        └── search.rs             # Provides the function to handle the search route.
```

## Development Tools

### Performance - Lighthouse

The easiest method of checking performance is to use Chromium's built-in auditing tool, Lighthouse. To run the test, open Developer Tools (usually F12) --> Lighthouse and click on the 'Generate Report' button at the bottom.

## Notes

### Known warnings

When running the build command, a warning appears. This is not an error and does not affect the security or performance of the application. They will be addressed soon in a future update.

```shell
warning: the following packages contain code that will be rejected by a future version of Rust: html5ever v0.23.0
note: to see what the problems were, use the option `--future-incompat-report`, or run `cargo report future-incompatibilities --id 2`
```

This warning just means that any dependencies or code using the `html5ever` code would be deprecated and rejected in future versions of the Rust language. So right now these dependencies can be used as these have not happened yet.

[⬅️ Go back to Home](./README.md)
