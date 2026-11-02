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
      case 'animations':
        cookie_dictionary['animation'] = select_tag.value || null
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
      engines.push(engine_checkbox.value)
    }
  })

  cookie_dictionary['engines'] = engines

  // Set the expiration date for the cookie to 1 year from the current date
  let expiration_date = new Date()
  expiration_date.setFullYear(expiration_date.getFullYear() + 1)

  // Save the cookie to the user's machine
  document.cookie = `appCookie=${JSON.stringify(
    cookie_dictionary,
  )}; expires=${expiration_date.toUTCString()}; path=/; SameSite=Lax`

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
  // Only the `appCookie` holds the user's saved preferences. Every other cookie
  // set on this origin (sessions, analytics, other self-hosted apps) is unrelated
  // and must never be parsed as JSON, otherwise a single unrelated cookie would
  // throw and abort the whole initialization in `initSettingsEventListeners()`.
  const app_cookie = document.cookie
    .split(';')
    .map((item) => item.trim())
    .find((item) => item.startsWith('appCookie='))

  if (!app_cookie) return

  let cookie_value
  try {
    cookie_value = JSON.parse(
      decodeURIComponent(app_cookie.slice('appCookie='.length)),
    )
  } catch (error) {
    // Never let a malformed cookie break the page; fall back to the server side
    // defaults and keep the rest of the initialization running.
    console.error('Error decoding the saved settings cookie:', error)
    return
  }

  if (!cookie_value || typeof cookie_value !== 'object') return

  let links = Array.from(document.querySelectorAll('link'))

  // A check to determine whether the animation link exists under the head tag or not.
  // If it does not exists then create and add a new animation link under the head tag
  // and update the other link tags href according to the settings provided by the user
  // via the UI. On the other hand if it does exist then just update all the link tags
  // href according to the settings provided by the user via the UI.
  if (!links.some((item) => item.href.includes('static/animations'))) {
    if (cookie_value['animation']) {
      let animation_link = document.createElement('link')
      animation_link.href = `static/animations/${cookie_value['animation']}.css`
      animation_link.rel = 'stylesheet'
      animation_link.type = 'text/css'
      document.querySelector('head').appendChild(animation_link)
    }
    // Loop through all link tags and update their href values to match the user's preferences
    links.forEach((item) => {
      if (item.href.includes('static/themes') && cookie_value['theme']) {
        item.href = `static/themes/${cookie_value['theme']}.css`
      } else if (
        item.href.includes('static/colorschemes') &&
        cookie_value['colorscheme']
      ) {
        item.href = `static/colorschemes/${cookie_value['colorscheme']}.css`
      }
    })
  } else {
    // Loop through all link tags and update their href values to match the user's preferences
    links.forEach((item) => {
      if (item.href.includes('static/themes') && cookie_value['theme']) {
        item.href = `static/themes/${cookie_value['theme']}.css`
      } else if (
        item.href.includes('static/colorschemes') &&
        cookie_value['colorscheme']
      ) {
        item.href = `static/colorschemes/${cookie_value['colorscheme']}.css`
      } else if (
        item.href.includes('static/animations') &&
        cookie_value['animation']
      ) {
        item.href = `static/animations/${cookie_value['animation']}.css`
      }
    })
    if (!cookie_value['animation']) {
      links
        .filter((item) => item.href.includes('static/animations'))
        .forEach((item) => item.remove())
    }
  }
}

function initSettingsEventListeners() {
  document
    .querySelectorAll('.settings_container .sidebar .btn')
    .forEach((tab) => tab.addEventListener('click', () => setActiveTab(tab)))

  document
    .querySelector('#save-settings')
    ?.addEventListener('click', setClientSettings)

  document
    .querySelector('#restore-settings')
    ?.addEventListener('submit', () => {
      setTimeout(() => window.location.reload(), 10)
    })

  document
    .querySelector('.select_all')
    ?.addEventListener('change', toggleAllSelection)

  // Restore the saved preferences last, and never let a bad cookie prevent the
  // event listeners above from being registered.
  try {
    getClientSettings()
  } catch (error) {
    console.error('Error restoring the saved settings:', error)
  }
}

if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initSettingsEventListeners)
} else {
  initSettingsEventListeners()
}
