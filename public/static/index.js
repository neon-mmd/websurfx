let search_box = document.querySelector('input');

function search_web() {
  window.location.href = `search?q=${search_box.value}`;
}

search_box.addEventListener('keyup', (e) => {
  if (e.key === 'Enter') {
    search_web();
  }
});

