// This function is executed when any page on the website finishes loading and
// this function retrieves the cookies if it is present on the user's machine.
// If it is available then the saved cookies is display in the cookies tab
// otherwise an appropriate message is displayed if it is not available.
document.addEventListener(
  'DOMContentLoaded',
  () => {
    try {
      let cookie = decodeURIComponent(document.cookie)
      document.querySelector('.cookies input').value =
        cookie !== '' ? cookie : 'No cookies have been saved on your system'
    } catch (error) {
      console.error('Error decoding cookie:', error)
      document.querySelector('.cookies input').value = 'Error decoding cookie'
    }
  },
  false
)
