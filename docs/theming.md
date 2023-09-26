# Colorschemes

## Built-in

By default `websurfx` comes with 9 colorschemes to choose from which can be easily chosen using the config file. To how to change colorschemes please view the [Configuration](https://github.com/neon-mmd/websurfx/wiki/configuration) section of the wiki.

## Custom

Creating coloschemes is as easy as it gets it requires the user to have a theme file name with the colorscheme in which every space should be replaced with a `-` (dash) and it should end with a `.css` file extension. After creating the file you need to add the following code with the `colors` you want:

```css
:root {
  --background-color: <background color>;
  --foreground-color: <foreground color (text color on the website) >;
  --color-one: <color 1>;
  --color-two: <color 2>;
  --color-three: <color 3>;
  --color-four: <color 4>;
  --color-five: <color 5>;
  --color-six: <color 6>;
  --color-seven: <color 7>;
}
```

> **Note**
> Please infer the theme file located under `public/static/themes` to better understand where each color is being used.

**Example of `catppuccin-mocha` colorscheme:**

```css
:root {
  --background-color: #1e1e2e;
  --foreground-color: #cdd6f4;
  --color-one: #45475a;
  --color-two: #f38ba8;
  --color-three: #a6e3a1;
  --color-four: #f9e2af;
  --color-five: #89b4fa;
  --color-six: #f5c2e7;
  --color-seven: #ffffff;
}
```

# Themes

## Built-in

By default `websurfx` comes with 1 theme to choose from which can be easily chosen using the config file. To how to change themes please view the [Configuration](https://github.com/neon-mmd/websurfx/wiki/configuration) section of the wiki.

## Custom

To write custom color scheme, it requires the user to have some knowledge of `css stylesheets`.

**Here is an example of `simple theme` (which we provide by default with the app) which will give the user a better idea on how to create a custom theme using it as a template:**

### General

```css
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
  background: var(--color-one);
}
```

### Styles for the index page

```css
.search-container {
  display: flex;
  flex-direction: column;
  gap: 5rem;
  justify-content: center;
  align-items: center;
}

.search-container div {
  display: flex;
}
```

### Styles for the search box and search button

```css
.search_bar {
  display: flex;
}

.search_bar input {
  padding: 1rem;
  width: 50rem;
  height: 3rem;
  outline: none;
  border: none;
  box-shadow: rgba(0, 0, 0, 1);
  background: var(--foreground-color);
}

.search_bar button {
  padding: 1rem;
  border-radius: 0;
  height: 3rem;
  display: flex;
  justify-content: center;
  align-items: center;
  outline: none;
  border: none;
  gap: 0;
  background: var(--background-color);
  color: var(--color-three);
  font-weight: 600;
  letter-spacing: 0.1rem;
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
  padding: 1rem 2rem;
  border-radius: 0.5rem;
  outline: none;
  border: none;
  text-transform: capitalize;
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
```

```css
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
  box-shadow: inset 0 0 100px 100px rgba(255, 255, 255, 0.1);
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
  margin: 0rem 7rem;
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

### Styles for the footer and header

```css
header {
  background: var(--background-color);
  width: 100%;
  display: flex;
  justify-content: right;
  align-items: center;
  padding: 1rem;
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

footer {
  background: var(--background-color);
  width: 100%;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

footer div span {
  font-size: 1.5rem;
  color: var(--4);
}

footer div {
  display: flex;
  gap: 1rem;
}

footer {
  background: var(--bg);
  width: 100%;
  padding: 1rem;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}
```

### Styles for the search page

```css
.results {
  width: 90%;
  display: flex;
  flex-direction: column;
  justify-content: space-around;
}

.results .search_bar {
  margin: 1rem 0;
}

.results_aggregated {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  margin: 2rem 0;
}

.results_aggregated .result {
  display: flex;
  flex-direction: column;
  margin-top: 1rem;
}

.results_aggregated .result h1 a {
  font-size: 1.5rem;
  color: var(--color-two);
  text-decoration: none;
  letter-spacing: 0.1rem;
}

.results_aggregated .result h1 a:hover {
  color: var(--color-five);
}

.results_aggregated .result h1 a:visited {
  color: var(--background-color);
}

.results_aggregated .result small {
  color: var(--color-three);
  font-size: 1.1rem;
  word-wrap: break-word;
  line-break: anywhere;
}

.results_aggregated .result p {
  color: var(--foreground-color);
  font-size: 1.2rem;
  margin-top: 0.3rem;
  word-wrap: break-word;
  line-break: anywhere;
}

.results_aggregated .result .upstream_engines {
  text-align: right;
  font-size: 1.2rem;
  padding: 1rem;
  color: var(--color-five);
}
```

### Styles for the 404 page

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

### Styles for the previous and next button on the search page

```css
.page_navigation {
  padding: 0 0 2rem 0;
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

### Styles for the about page

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

### Styles for the Settings Page

This part is only available right now in the **rolling/edge/unstable** version

```css
.settings_container {
  display: flex;
  justify-content: space-around;
  width: 80dvw;
}

.settings h1 {
  color: var(--color-two);
  font-size: 2.5rem;
}

.settings hr {
  border-color: var(--color-three);
  margin: 0.3rem 0 1rem 0;
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
  font-weight: bold;
  margin-bottom: 0.5rem;
  color: var(--foreground-color);
  text-transform: capitalize;
  gap: 1.5rem;
}

.settings_container .sidebar .btn {
  padding: 0.5rem;
  border-radius: 0.5rem;
}

.settings_container .sidebar .btn.active {
  background-color: var(--color-two);
}

.settings_container .main_container {
  width: 70%;
  border-left: 1.5px solid var(--color-three);
  padding-left: 3rem;
}

.settings_container .tab {
  display: none;
}

.settings_container .tab.active {
  display: flex;
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
  background-color: var(--background-color);
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
  gap: 1rem;
  padding: 1rem 0;
}

.settings_container .engines .toggle_btn {
  color: var(--foreground-color);
  font-size: 1.5rem;
  display: flex;
  gap: 0.5rem;
  align-items: center;
}

.settings_container .engines hr {
  margin: 0;
}

.settings_container .cookies input {
  margin: 1rem 0rem;
}
```

### Styles for the Toggle Button

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
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--background-color);
  -webkit-transition: 0.4s;
  transition: 0.4s;
}

.slider:before {
  position: absolute;
  content: '';
  height: 2.6rem;
  width: 2.6rem;
  left: 0.4rem;
  bottom: 0.4rem;
  background-color: var(--foreground-color);
  -webkit-transition: 0.4s;
  transition: 0.4s;
}

input:checked + .slider {
  background-color: var(--color-three);
}

input:focus + .slider {
  box-shadow: 0 0 1px var(--color-three);
}

input:checked + .slider:before {
  -webkit-transform: translateX(2.6rem);
  -ms-transform: translateX(2.6rem);
  transform: translateX(2.6rem);
}

/* Rounded sliders */
.slider.round {
  border-radius: 3.4rem;
}

.slider.round:before {
  border-radius: 50%;
}
```

[⬅️ Go back to Home](./README.md)
