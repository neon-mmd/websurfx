// This function handles the toggling of selections of all upstream search engines 
// options in the settings page under the tab engines.
function toggleAllSelection() {
  document
    .querySelectorAll('.engine')
    .forEach(
      (engine_checkbox) =>
        (engine_checkbox.checked =
          document.querySelector('.select_all').checked)
    )
}

// This function adds the functionality to sidebar buttons to only show settings 
// related to that tab.
function setActiveTab(current_tab) {
  document
    .querySelectorAll('.tab')
    .forEach((tab) => tab.classList.remove('active'))
  document
    .querySelectorAll('.btn')
    .forEach((tab) => tab.classList.remove('active'))
  current_tab.classList.add('active')
  document
    .querySelector(`.${current_tab.innerText.toLowerCase().replace(' ', '_')}`)
    .classList.add('active')
}

// This function adds the functionality to save all the user selected preferences
// to be saved in a cookie on the users machine.
function setClientSettings() {
  let cookie_dictionary = new Object()
  document.querySelectorAll('select').forEach((select_tag) => {
    if (select_tag.name === 'themes') {
      cookie_dictionary['theme'] = select_tag.value
    } else if (select_tag.name === 'colorschemes') {
      cookie_dictionary['colorscheme'] = select_tag.value
    }
  })
  let engines = []
  document.querySelectorAll('.engine').forEach((engine_checkbox) => {
    if (engine_checkbox.checked === true) {
      engines.push(engine_checkbox.parentNode.parentNode.innerText.trim())
    }
  })
  cookie_dictionary['engines'] = engines
  let expiration_date = new Date()
  expiration_date.setFullYear(expiration_date.getFullYear() + 1)
  document.cookie = `appCookie=${JSON.stringify(
    cookie_dictionary
  )}; expires=${expiration_date.toUTCString()}`

  document.querySelector('.message').innerText =
    '✅ The settings have been saved sucessfully!!'

  setTimeout(() => {
    document.querySelector('.message').innerText = ''
  }, 10000)
}

// This functions gets the saved cookies if it is present on the user's machine If it 
// is available then it is parsed and converted to an object which is then used to 
// retrieve the preferences that the user had selected previously and is then loaded in the 
// website otherwise the function does nothing and the default server side settings are loaded.
function getClientSettings() {
  let cookie = decodeURIComponent(document.cookie)

  if (cookie !== '') {
    let cookie_value = decodeURIComponent(document.cookie)
      .split(';')
      .map((item) => item.split('='))
      .reduce((acc, [_, v]) => (acc = JSON.parse(v)) && acc, {})

    let links = Array.from(document.querySelectorAll('link')).forEach(
      (item) => {
        if (item.href.includes('static/themes')) {
          item.href = `static/themes/${cookie_value['theme']}.css`
        } else if (item.href.includes('static/colorschemes')) {
          item.href = `static/colorschemes/${cookie_value['colorscheme']}.css`
        }
      }
    )
  }
}
