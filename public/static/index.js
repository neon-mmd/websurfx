let search_box = document.querySelector('input')
function search_web() {
  window.location = `search?q=${search_box.value}`
}

search_box.addEventListener('keyup', (e) => {
  if (e.keyCode === 13) {
    search_web()
  }
})
