# Colorschemes

## Built-in

By default `websurfx` comes with 9 colorschemes to choose from which can be easily chosen using the config file. To how to change colorschemes please view the [Configuration](https://github.com/neon-mmd/websurfx/wiki/configuration) section of the wiki.

## Custom

Creating coloschemes is as easy as it gets it requires the user to have a theme file name with the colorscheme in which every space should be replaced with a `-` (dash) and it should end with a `.css` file extension. After creating the file you need to add the following code with the `colors` you want:

``` css
:root{
  --bg: <background color>;
  --fg: <foreground color (text color)>;
  --1: <color 1>;
  --2: <color 2>;
  --3: <color 3>;
  --4: <color 4>;
  --5: <color 5>;
  --6: <color 6>;
  --7: <color 7>;
}
```

> **Note**
> Please infer the theme file located under `public/static/themes` to better understand where each color is being used.

**Example of `catppuccin-mocha` colorscheme:**

``` css
:root {
  --bg: #1e1e2e;
  --fg: #cdd6f4;
  --1: #45475a;
  --2: #f38ba8;
  --3: #a6e3a1;
  --4: #f9e2af;
  --5: #89b4fa;
  --6: #f5c2e7;
  --7: #ffffff;
}
```

# Themes

## Built-in

By default `websurfx` comes with 1 theme to choose from which can be easily chosen using the config file. To how to change themes please view the [Configuration](https://github.com/neon-mmd/websurfx/wiki/configuration) section of the wiki.

## Custom 

To write custom color scheme, it requires the user to have some knowledge of `css stylesheets`. 

**Here is an example of `simple theme` (which we provide by default with the app) which will give the user a better idea on how to create a custom theme using it as a template:**

### General
``` css
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
  background: var(--1);
}
```
### Styles for the index page
``` css
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
``` css
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
  background: var(--fg);
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
  background: var(--bg);
  color: var(--3);
  font-weight: 600;
  letter-spacing: 0.1rem;
}

.search_bar button:active,
.search_bar button:hover {
  filter: brightness(1.2);
}
```
### Styles for the footer and header
``` css
header {
  background: var(--bg);
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
  color: var(--2);
  text-transform: capitalize;
  letter-spacing: 0.1rem;
}

header ul li a {
  font-weight: 600;
}

header ul li a:hover,
footer ul li a:hover {
  color: var(--5);
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
``` css
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
  color: var(--2);
  text-decoration: none;
  letter-spacing: 0.1rem;
}

.results_aggregated .result h1 a:hover {
  color: var(--5);
}

.results_aggregated .result h1 a:visited {
  color: var(--bg);
}

.results_aggregated .result small {
  color: var(--3);
  font-size: 1.1rem;
  word-wrap: break-word;
  line-break: anywhere;
}

.results_aggregated .result p {
  color: var(--fg);
  font-size: 1.2rem;
  margin-top: 0.3rem;
  word-wrap: break-word;
  line-break: anywhere;
}

.results_aggregated .result .upstream_engines {
  text-align: right;
  font-size: 1.2rem;
  padding: 1rem;
  color: var(--5);
}
```

### Styles for the 404 page 

``` css
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
  color: var(--2);
  text-decoration: none;
}

.error_content p a:hover {
  color: var(--5);
}
```
### Styles for the previous and next button on the search page
``` css
.page_navigation {
  padding: 0 0 2rem 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page_navigation button {
  background: var(--bg);
  color: var(--fg);
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
.about-container article{
    font-size: 1.5rem;
    color:var(--fg);
    padding-bottom: 10px;
  }

.about-container article h1{
    color: var(--2);
    font-size: 2.8rem;
  }

.about-container article div{
    padding-bottom: 15px;
  }

.about-container a{
  color:var(--3);
}

.about-container article h2{
  color: var(--3);
  font-size: 1.8rem;
  padding-bottom: 10px;
}

.about-container p{
  color:var(--fg);
  font-size:  1.6rem;
  padding-bottom: 10px;
}

.about-container h3{
  font-size: 1.5rem;
}
```

[⬅️  Go back to Home](./README.md)
