/**
 * Selects the input element for the search box
 * @type {HTMLInputElement}
 */
const searchBox = document.querySelector('input')

/**
 * Redirects the user to the search results page with the query parameter
 */
function searchWeb() {
    const query = searchBox.value.trim()
    try {
        let safeSearchLevel = document.querySelector('.search_options select').value
        if (query) {
            window.location.href = `search?q=${encodeURIComponent(
                query,
            )}&safesearch=${encodeURIComponent(safeSearchLevel)}`
        }
    } catch (error) {
        if (query) {
            window.location.href = `search?q=${encodeURIComponent(query)}`
        }
    }
}

/**
 * Listens for the 'Enter' key press event on the search box and calls the searchWeb function
 * @param {KeyboardEvent} e - The keyboard event object
 */
searchBox.addEventListener('keyup', (e) => {
    if (e.key === 'Enter') {
        searchWeb()
    }
})
