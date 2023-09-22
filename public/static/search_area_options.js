document.addEventListener(
    'DOMContentLoaded',
    () => {
        let url = new URL(window.location)
        let searchParams = url.searchParams

        let safeSearchLevel = searchParams.get('safesearch')

        if (
            safeSearchLevel >= 0 &&
            safeSearchLevel <= 2 &&
            safeSearchLevel !== null
        ) {
            document.querySelector('.search_options select').value = safeSearchLevel
        }
    },
    false,
)
