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
      document.querySelector('.cookies input').value = cookie.length
        ? cookie
        : 'No cookies have been saved on your system'
    } catch (error) {
      // If there is an error decoding the cookie, log the error to the console
      // and display an error message in the input field
      console.error('Error decoding cookie:', error)
      document.querySelector('.cookies input').value = 'Error decoding cookie'
    }
  },
  false,
)
