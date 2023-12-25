# Theming

## Colorschemes

### Built-in

By default `websurfx` comes with 12 colorschemes to choose from which can be easily chosen using the config file or via the settings page on the website.

> To how to change colorschemes using the config file. See: [**Configuration**](https://github.com/neon-mmd/websurfx/wiki/configuration)

### Custom

To write a custom theme for the website, you will first need to create a new file under the `public/static/themes` folder with name of the theme containing each word seperated with a hyphen (**-**). Then after that edit the newly created file as required with new css code.

Creating coloschemes is as easy as it gets it requires the user to have a colorscheme file name with the name of the colorscheme that is to be provided in which every space should be replaced with a `-` (dash) and it should end with a `.css` file extension. After creating the file you need to add the following code with the `colors` you want to include:

```css
:root {
  --background-color: <background color>;
  --foreground-color: <foreground color (text color on the website) >;
  --logo-color: <logo color
    (the color of the logo svg image on the website homepage) >;
  --color-one: <color 1>;
  --color-two: <color 2>;
  --color-three: <color 3>;
  --color-four: <color 4>;
  --color-five: <color 5>;
  --color-six: <color 6>;
  --color-seven: <color 7>;
}
```

> [!Note]
> Please infer the theme file located under `public/static/themes` to better understand where each color is being used.

**Example of `catppuccin-mocha` colorscheme:**

```css
:root {
  --background-color: #1e1e2e;
  --foreground-color: #cdd6f4;
  --logo-color: #f5c2e7;
  --color-one: #45475a;
  --color-two: #f38ba8;
  --color-three: #a6e3a1;
  --color-four: #f9e2af;
  --color-five: #89b4fa;
  --color-six: #f5c2e7;
  --color-seven: #ffffff;
}
```

## Themes

### Built-in

By default `websurfx` comes with 1 theme to choose from which can be easily chosen using the config file or via the settings page on the website.

> To how to change themes using the config file. See: [**Configuration**](https://github.com/neon-mmd/websurfx/wiki/configuration)

### Custom

> This section expects the user to have some knowledge of `css`.

To write a custom theme for the website, you will first need to create a new file under the `public/static/themes` folder with name of the theme containing each word seperated with a hyphen (**-**). Then after that edit the newly created file as required with new css code.

Here is an example of `simple theme` (which we provide by default with the app) which will give you a better idea on how you can create your own custom theme for the website:

#### General

```css
@font-face {
  font-family: Rubik;
  src: url('https://fonts.googleapis.com/css2?family=Rubik:wght@400;500;600;700;800&display=swap');
  fallback: sans-serif;
}

* {
  padding: 0;
  margin: 0;
  box-sizing: border-box;
}

html {
  font-size: 62.5%;
}

body {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  align-items: center;
  height: 100vh;
  font-family: Rubik, sans-serif;
  background-color: var(--background-color);
}

/* enforce font for buttons */
button {
  font-family: Rubik, sans-serif;
}
```

#### Styles for the index page

```css
.search-container {
  display: flex;
  flex-direction: column;
  gap: 5rem;
  justify-content: center;
  align-items: center;
}

.search-container svg {
  color: var(--logo-color);
}

.search-container div {
  display: flex;
}
```

#### Styles for the search box and search button

```css
.search_bar {
  display: flex;
  gap: 10px;
  align-items: center;
}

.search_bar input {
  border-radius: 6px;
  padding: 2.6rem 2.2rem;
  width: 50rem;
  height: 3rem;
  outline: none;
  border: none;
  box-shadow: rgb(0 0 0 / 1);
  background-color: var(--color-one);
  color: var(--foreground-color);
  outline-offset: 3px;
  font-size: 1.6rem;
}

.search_bar input:focus {
  outline: 2px solid var(--foreground-color);
}

.search_bar input::placeholder {
  color: var(--foreground-color);
  opacity: 1;
}

.search_bar button {
  padding: 2.6rem 3.2rem;
  border-radius: 6px;
  height: 3rem;
  display: flex;
  justify-content: center;
  align-items: center;
  outline-offset: 3px;
  outline: 2px solid transparent;
  border: none;
  transition: 0.1s;
  gap: 0;
  background-color: var(--color-six);
  color: var(--background-color);
  font-weight: 600;
  letter-spacing: 0.1rem;
}

.search_bar button:active {
  outline: 2px solid var(--color-three);
}

.search_bar button:active,
.search_bar button:hover {
  filter: brightness(1.2);
}

.search_area .search_options {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.search_area .search_options select {
  margin: 0.7rem 0;
  width: 20rem;
  background-color: var(--color-one);
  color: var(--foreground-color);
  padding: 1.2rem 2rem;
  border-radius: 0.5rem;
  outline-offset: 3px;
  outline: 2px solid transparent;
  border: none;
  text-transform: capitalize;
}

.search_area .search_options select:active,
.search_area .search_options select:hover {
  outline: 2px solid var(--color-three);
}

.search_area .search_options option:hover {
  background-color: var(--color-one);
}

.result_not_found {
  display: flex;
  flex-direction: column;
  font-size: 1.5rem;
  color: var(--foreground-color);
}

.result_not_found p {
  margin: 1rem 0;
}

.result_not_found ul {
  margin: 1rem 0;
}

.result_not_found img {
  width: 40rem;
}

/* styles for the error box */
.error_box .error_box_toggle_button {
  background: var(--foreground-color);
}

.error_box .dropdown_error_box {
  position: absolute;
  display: none;
  flex-direction: column;
  background: var(--background-color);
  border-radius: 0;
  margin-left: 2rem;
  min-height: 20rem;
  min-width: 22rem;
}

.error_box .dropdown_error_box.show {
  display: flex;
}

.error_box .dropdown_error_box .error_item,
.error_box .dropdown_error_box .no_errors {
  display: flex;
  align-items: center;
  color: var(--foreground-color);
  letter-spacing: 0.1rem;
  padding: 1rem;
  font-size: 1.2rem;
}

.error_box .dropdown_error_box .error_item {
  justify-content: space-between;
}

.error_box .dropdown_error_box .no_errors {
  min-height: 18rem;
  justify-content: center;
}

.error_box .dropdown_error_box .error_item:hover {
  box-shadow: inset 0 0 100px 100px rgb(255 255 255 / 0.1);
}

.error_box .error_item .severity_color {
  width: 1.2rem;
  height: 1.2rem;
}

.results .result_disallowed,
.results .result_filtered,
.results .result_engine_not_selected {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 10rem;
  font-size: 2rem;
  color: var(--foreground-color);
  margin: 0 7rem;
}

.results .result_disallowed .user_query,
.results .result_filtered .user_query,
.results .result_engine_not_selected .user_query {
  color: var(--background-color);
  font-weight: 300;
}

.results .result_disallowed img,
.results .result_filtered img,
.results .result_engine_not_selected img {
  width: 30rem;
}

.results .result_disallowed div,
.results .result_filtered div,
.results .result_engine_not_selected div {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  line-break: strict;
}
```

#### Styles for the footer and header

```css
header {
  width: 100%;
  background: var(--background-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2rem 3rem;
}

footer {
  width: 100%;
  background: var(--background-color);
  display: flex;
  align-items: center;
  padding: 1.7rem 1.7rem 4rem;
  gap: 1.8rem;
  flex-direction: column;
  justify-content: center;
}

header h1 a {
  text-transform: capitalize;
  text-decoration: none;
  color: var(--foreground-color);
  letter-spacing: 0.1rem;
}

header ul,
footer ul {
  list-style: none;
  display: flex;
  justify-content: space-around;
  align-items: center;
  font-size: 1.5rem;
  gap: 2rem;
}

header ul li a,
footer ul li a,
header ul li a:visited,
footer ul li a:visited {
  text-decoration: none;
  color: var(--color-two);
  text-transform: capitalize;
  letter-spacing: 0.1rem;
}

header ul li a {
  font-weight: 600;
}

header ul li a:hover,
footer ul li a:hover {
  color: var(--color-five);
}

footer div span {
  font-size: 1.5rem;
  color: var(--color-four);
}

footer div {
  display: flex;
  gap: 1rem;
}
```

#### Styles for the search page

```css
.results {
  width: 90%;
  display: flex;
  flex-direction: column;
  justify-content: space-around;
  gap: 1rem;
}

.result {
  gap: 1rem;
}

.results .search_bar {
  margin: 1rem 0;
}

.results_aggregated {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  margin: 2rem 0;
  content-visibility: auto;
}

.results_aggregated .result {
  display: flex;
  flex-direction: column;
  margin-top: 1rem;
}

.results_aggregated .result h1 a {
  font-size: 1.7rem;
  font-weight: normal;
  color: var(--color-two);
  text-decoration: none;
}

.results_aggregated .result h1 a:hover {
  color: var(--color-five);
}

.results_aggregated .result h1 a:visited {
  color: var(--background-color);
}

.results_aggregated .result small {
  color: var(--color-three);
  font-size: 1.3rem;
  word-wrap: break-word;
  line-break: anywhere;
}

.results_aggregated .result p {
  color: var(--foreground-color);
  font-size: 1.4rem;
  line-height: 2.4rem;
  margin-top: 0.3rem;
  word-wrap: break-word;
  line-break: anywhere;
}

.results_aggregated .result .upstream_engines {
  text-align: right;
  font-size: 1.2rem;
  padding: 1rem;
  color: var(--color-five);
  display: flex;
  gap: 1rem;
  justify-content: right;
}
```

#### Styles for the 404 page

```css
.error_container {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  gap: 5rem;
}

.error_container img {
  width: 30%;
}

.error_content {
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 1rem;
}

.error_content h1,
.error_content h2 {
  letter-spacing: 0.1rem;
}

.error_content h1 {
  font-size: 3rem;
}

.error_content h2 {
  font-size: 2rem;
}

.error_content p {
  font-size: 1.2rem;
}

.error_content p a,
.error_content p a:visited {
  color: var(--color-two);
  text-decoration: none;
}

.error_content p a:hover {
  color: var(--color-five);
}
```

#### Styles for the previous and next button on the search page

```css
.page_navigation {
  padding: 0 0 2rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page_navigation button {
  background: var(--background-color);
  color: var(--foreground-color);
  padding: 1rem;
  border-radius: 0.5rem;
  outline: none;
  border: none;
}

.page_navigation button:active {
  filter: brightness(1.2);
}
```

#### Styles for the about page

This part is only available right now in the **rolling/edge/unstable** version

```css
.about-container article {
  font-size: 1.5rem;
  color: var(--foreground-color);
  padding-bottom: 10px;
}

.about-container article h1 {
  color: var(--color-two);
  font-size: 2.8rem;
}

.about-container article div {
  padding-bottom: 15px;
}

.about-container a {
  color: var(--color-three);
}

.about-container article h2 {
  color: var(--color-three);
  font-size: 1.8rem;
  padding-bottom: 10px;
}

.about-container p {
  color: var(--foreground-color);
  font-size: 1.6rem;
  padding-bottom: 10px;
}

.about-container h3 {
  font-size: 1.5rem;
}

.about-container {
  width: 80%;
}
```

#### Styles for the Settings Page

This part is only available right now in the **rolling/edge/unstable** version

```css
.settings_container {
  display: flex;
  justify-content: space-around;
  width: 80dvw;
  margin: 5rem 0;
}

.settings h1 {
  color: var(--color-two);
  font-size: 2.5rem;
}

.settings > h1 {
  margin-bottom: 4rem;
  margin-left: 2rem;
}

.settings hr {
  border-color: var(--color-three);
  margin: 0.3rem 0 1rem;
}

.settings > hr {
  margin-left: 2rem;
}

.settings_container .sidebar {
  width: 30%;
  cursor: pointer;
  font-size: 2rem;
  display: flex;
  flex-direction: column;
  margin-right: 0.5rem;
  margin-left: -0.7rem;
  padding: 0.7rem;
  border-radius: 5px;
  margin-bottom: 0.5rem;
  color: var(--foreground-color);
  text-transform: capitalize;
  gap: 1.5rem;
}

.settings_container .sidebar .btn {
  padding: 2rem;
  border-radius: 0.5rem;
  outline-offset: 3px;
  outline: 2px solid transparent;
}

.settings_container .sidebar .btn:active {
  outline: 2px solid var(--color-two);
}

.settings_container .sidebar .btn:not(.active):hover {
  color: var(--color-two);
}

.settings_container .sidebar .btn.active {
  background-color: var(--color-two);
  color: var(--background-color);
}

.settings_container .main_container {
  width: 70%;
  border-left: 1.5px solid var(--color-three);
  padding-left: 3rem;
  border: none;
}

.settings_container .tab {
  display: none;
}

.settings_container .tab.active {
  display: flex;
  gap: 1.2rem;
  flex-direction: column;
  justify-content: space-around;
}

.settings_container button {
  margin-top: 1rem;
  padding: 1rem 2rem;
  font-size: 1.5rem;
  background: var(--color-three);
  color: var(--background-color);
  border-radius: 0.5rem;
  border: 2px solid transparent;
  font-weight: bold;
  transition: all 0.1s ease-out;
  cursor: pointer;
  box-shadow: 5px 5px;
  outline: none;
}

.settings_container button:active {
  box-shadow: none;
  translate: 5px 5px;
}

.settings_container .main_container .message {
  font-size: 1.5rem;
  color: var(--foreground-color);
}

.settings_container .tab h3 {
  font-size: 2rem;
  font-weight: bold;
  color: var(--color-four);
  margin-top: 1.5rem;
  text-transform: capitalize;
}

.settings_container .tab .description {
  font-size: 1.5rem;
  margin-bottom: 0.5rem;
  color: var(--foreground-color);
}

.settings_container .user_interface select,
.settings_container .general select {
  margin: 0.7rem 0;
  width: 20rem;
  background-color: var(--color-one);
  color: var(--foreground-color);
  padding: 1rem 2rem;
  border-radius: 0.5rem;
  outline: none;
  border: none;
  text-transform: capitalize;
}

.settings_container .user_interface option:hover,
.settings_container .general option:hover {
  background-color: var(--color-one);
}

.settings_container .engines .engine_selection {
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 1rem 0;
  margin-bottom: 2rem;
  gap: 2rem;
}

.settings_container .engines .toggle_btn {
  color: var(--foreground-color);
  font-size: 1.5rem;
  display: flex;
  align-items: center;
  border-radius: 100px;
  gap: 1.5rem;
  letter-spacing: 1px;
}

.settings_container .engines hr {
  margin: 0;
}

.settings_container .cookies input {
  margin: 1rem 0;
}
```

#### Styles for the Toggle Button

This part is only available right now in the **rolling/edge/unstable** version

```css
/* The switch - the box around the slider */
.switch {
  position: relative;
  display: inline-block;
  width: 6rem;
  height: 3.4rem;
}

/* Hide default HTML checkbox */
.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

/* The slider */
.slider {
  position: absolute;
  cursor: pointer;
  inset: 0;
  background-color: var(--foreground-color);
  transition: 0.2s;
  outline-offset: 3px;
  outline: 2px solid transparent;
}

.slider:active {
  outline: 2px solid var(--foreground-color);
}

.slider::before {
  position: absolute;
  content: '';
  height: 2.6rem;
  width: 2.6rem;
  left: 0.4rem;
  bottom: 0.4rem;
  background-color: var(--background-color);
  transition: 0.2s;
}

input:checked + .slider {
  background-color: var(--color-three);
}

input:focus + .slider {
  box-shadow: 0 0 1px var(--color-three);
}

input:checked + .slider::before {
  transform: translateX(2.6rem);
}

/* Rounded sliders */
.slider.round {
  border-radius: 3.4rem;
}

.slider.round::before {
  border-radius: 50%;
}
```

## Animations

### Built-in

By default `websurfx` comes with 1 animation to choose from which can be easily chosen using the config file or via the settings page on the website.

> To how to change animations using the config file. See: [**Configuration**](https://github.com/neon-mmd/websurfx/wiki/configuration)

### Custom

To write custom animation, it requires the user to have some knowledge of `themes` and the `HTML of the page for which the animation is being provided for`.

The animations can be of 2 categories:

- Theme specific animations
- Universal animations

#### Theme Specific Animations

These animations can only be used with a specific theme and should not be used with other themes otherwise it either won't look good or won't work at all or would work partially.

Here is an example of `simple-frosted-glow` animation for the `simple theme` (which we provide by default with the app) which will give you a better idea on how to create a custom animation for a specific theme:

```css
.results_aggregated .result {
  margin: 1rem;
  padding: 1rem;
  border-radius: 1rem;
}

.results_aggregated .result:hover {
  box-shadow:
    inset 0 0 3rem var(--color-two),
    inset 0 0 6rem var(--color-five),
    inset 0 0 9rem var(--color-three),
    0 0 0.25rem var(--color-two),
    0 0 0.5rem var(--color-five),
    0 0 0.75rem var(--color-three);
}
```

#### Universal Animations

These animations are independent of the theme being used and can be used with all the themes.

Here is an example of `text-tilt` animation which will give you an idea on how to create universal animations for the search engine website.

```css
.results_aggregated .result:hover {
  transform: skewX(10deg);
}
```

> [!Note]
> 1. The above-mentioned examples of animations was covered for the search page of the search engine website. While the same way of creating custom animations can also be done for other pages also.
> 2. While the naming the file for the new theme file. Follow the following naming conventions:
>    1. If the animation is theme specfic then name of the animation file should look like this:
>       `<name of the theme which these animation is for><seperated by a hyphen or dash><name of the animation with whitespaces replaced with hyphens>`
>       **For example:**
>       If the animation to make search results frosty glow on hover was to be created for the `simple` theme then the name of the file would look something like this: 
>       `simple-frosted-glow`
>       Where `simple` is the name of the theme the animation targets and `frosted-glow` is the name of the animation where each word has been seperated by a hyphen.
>    2. If the animation is not theme specfic (univeral theme) then name of the animation file should look like this:
>       `<name of the animation with whitespaces replaced with hyphens>`
>       **For example:**
>       If the animation to make search results text tilt on hover was to be created then the name of the file would look something like this: 
>       `text-tilt`
>       Where `text-tilt` is the name of the animation where each word has been seperated by a hyphen. (While naming the files for these types of themes, You do not need to add a theme name in frontend of the file name.).


[⬅️ Go back to Home](./README.md)
