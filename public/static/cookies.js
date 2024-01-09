/**
 * This functions gets the saved cookies if it is present on the user's machine If it
 * is available then it is parsed and converted to an object which is then used to
 * retrieve the preferences that the user had selected previously and is then loaded
 * and used for displaying the user provided settings by setting them as the selected
 * options in the settings page.
 *
 * @function
 * @param {string} cookie - It takes the client settings cookie as a string.
 * @returns {void}
 */
function setClientSettingsOnPage(cookie) {
  let cookie_value = cookie
    .split(';')
    .map((item) => item.split('='))
    .reduce((acc, [_, v]) => (acc = JSON.parse(v)) && acc, {})

  // Loop through all select tags and add their values to the cookie dictionary
  document.querySelectorAll('select').forEach((select_tag) => {
    switch (select_tag.name) {
      case 'themes':
        select_tag.value = cookie_value['theme']
        break
      case 'colorschemes':
        select_tag.value = cookie_value['colorscheme']
        break
      case 'animations':
        select_tag.value = cookie_value['animation']
        break
      case 'safe_search_levels':
        select_tag.value = cookie_value['safe_search_level']
        break
    }
  })
  let engines = document.querySelectorAll('.engine')
  let engines_cookie = cookie_value['engines']

  if (engines_cookie.length === engines.length) {
    document.querySelector('.select_all').checked = true
    engines.forEach((engine_checkbox) => {
      engine_checkbox.checked = true
    })
  } else {
    engines.forEach((engines_checkbox) => {
      engines_checkbox.checked = false
    })
    engines_cookie.forEach((engine_name) => {
      engines.forEach((engine_checkbox) => {
        if (
          engine_checkbox.parentNode.parentNode.innerText.trim() ===
          engine_name.trim()
        ) {
          engine_checkbox.checked = true
        }
      })
    })
  }
}

/**
 * This function is executed when any page on the website finishes loading and
 * this function retrieves the cookies if it is present on the user's machine.
 * If it is available then the saved cookies is display in the cookies tab
 * otherwise an appropriate message is displayed if it is not available.
 *
 * @function
 * @listens DOMContentLoaded
 * @returns {void}
 */
document.addEventListener(
  'DOMContentLoaded',
  () => {
    try {
      // Decode the cookie value
      let cookie = decodeURIComponent(document.cookie)
      // Set the value of the input field to the decoded cookie value if it is not empty
      // Otherwise, display a message indicating that no cookies have been saved on the user's system
      if (cookie.length) {
        document.querySelector('.cookies input').value = cookie
        // This function displays the user provided settings on the settings page.
        setClientSettingsOnPage(cookie)
      } else {
        document.querySelector('.cookies input').value =
          'No cookies have been saved on your system'
      }
    } catch (error) {
      // If there is an error decoding the cookie, log the error to the console
      // and display an error message in the input field
      console.error('Error decoding cookie:', error)
      document.querySelector('.cookies input').value = 'Error decoding cookie'
    }
  },
  false,
)
