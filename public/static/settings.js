/**
 * This function handles the toggling of selections of all upstream search engines
 * options in the settings page under the tab engines.
 */
function toggleAllSelection() {
  document
    .querySelectorAll('.engine')
    .forEach(
      (engine_checkbox) =>
        (engine_checkbox.checked =
          document.querySelector('.select_all').checked),
    )
}

/**
 * This function adds the functionality to sidebar buttons to only show settings
 * related to that tab.
 * @param {HTMLElement} current_tab - The current tab that was clicked.
 */
function setActiveTab(current_tab) {
  // Remove the active class from all tabs and buttons
  document
    .querySelectorAll('.tab')
    .forEach((tab) => tab.classList.remove('active'))
  document
    .querySelectorAll('.btn')
    .forEach((tab) => tab.classList.remove('active'))

  // Add the active class to the current tab and its corresponding settings
  current_tab.classList.add('active')
  document
    .querySelector(`.${current_tab.innerText.toLowerCase().replace(' ', '_')}`)
    .classList.add('active')
}

/**
 * This function adds the functionality to save all the user selected preferences
 * to be saved in a cookie on the users machine.
 */
function setClientSettings() {
  // Create an object to store the user's preferences
  let cookie_dictionary = new Object()

  // Loop through all select tags and add their values to the cookie dictionary
  document.querySelectorAll('select').forEach((select_tag) => {
    switch (select_tag.name) {
      case 'themes':
        cookie_dictionary['theme'] = select_tag.value
        break
      case 'colorschemes':
        cookie_dictionary['colorscheme'] = select_tag.value
        break
      case 'safe_search_levels':
        cookie_dictionary['safe_search_level'] = Number(select_tag.value)
        break
    }
  })

  // Loop through all engine checkboxes and add their values to the cookie dictionary
  let engines = []

  document.querySelectorAll('.engine').forEach((engine_checkbox) => {
    if (engine_checkbox.checked) {
      engines.push(engine_checkbox.parentNode.parentNode.innerText.trim())
    }
  })

  cookie_dictionary['engines'] = engines

  // Set the expiration date for the cookie to 1 year from the current date
  let expiration_date = new Date()
  expiration_date.setFullYear(expiration_date.getFullYear() + 1)

  // Save the cookie to the user's machine
  document.cookie = `appCookie=${JSON.stringify(
    cookie_dictionary,
  )}; expires=${expiration_date.toUTCString()}`

  // Display a success message to the user
  document.querySelector('.message').innerText =
    '✅ The settings have been saved sucessfully!!'

  // Clear the success message after 10 seconds
  setTimeout(() => {
    document.querySelector('.message').innerText = ''
  }, 10000)
}

/**
 * This functions gets the saved cookies if it is present on the user's machine If it
 * is available then it is parsed and converted to an object which is then used to
 * retrieve the preferences that the user had selected previously and is then loaded in the
 * website otherwise the function does nothing and the default server side settings are loaded.
 */
function getClientSettings() {
  // Get the appCookie from the user's machine
  let cookie = decodeURIComponent(document.cookie)

  // If the cookie is not empty, parse it and use it to set the user's preferences
  if (cookie.length) {
    let cookie_value = cookie
      .split(';')
      .map((item) => item.split('='))
      .reduce((acc, [_, v]) => (acc = JSON.parse(v)) && acc, {})

    // Loop through all link tags and update their href values to match the user's preferences
    Array.from(document.querySelectorAll('link')).forEach((item) => {
      if (item.href.includes('static/themes')) {
        item.href = `static/themes/${cookie_value['theme']}.css`
      } else if (item.href.includes('static/colorschemes')) {
        item.href = `static/colorschemes/${cookie_value['colorscheme']}.css`
      }
    })
  }
}
