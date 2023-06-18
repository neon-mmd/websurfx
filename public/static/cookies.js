// This function is executed when any page on the website finsihes loading and
// this function retrieves the cookies if it is present on the user's machine. 
// If it is available then the saved cookies is display in the cookies tab 
// otherwise an appropriate message is displayed if it is not available.
document.addEventListener(
  'DOMContentLoaded',
  () => {
    let cookie = decodeURIComponent(document.cookie)
    if (cookie !== '') {
      document.querySelector('.cookies input').value = cookie
    } else {
      document.querySelector('.cookies input').value =
        'No cookies has been saved on your system'
    }
  },
  false
)
