/**
* A function that clears the search input text when the clear button is clicked.
*/
function clearSearchText() {
    const input = document.querySelector('.search_bar > input')
    if (input) input.value = ''
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        document.querySelector('.clear_search_btn')?.addEventListener('click', clearSearchText)
    })
} else {
    document.querySelector('.clear_search_btn')?.addEventListener('click', clearSearchText)
}
