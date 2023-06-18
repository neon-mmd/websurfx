// After the settings page finishes loading 
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
