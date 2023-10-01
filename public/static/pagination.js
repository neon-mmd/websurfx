/**
 * Navigates to the next page by incrementing the current page number in the URL query string.
 * @returns {void}
 */
function navigate_forward() {
    let url = new URL(window.location);
    let searchParams = url.searchParams;

    let q = searchParams.get('q');
    let page = parseInt(searchParams.get('page'));

    if (isNaN(page)) {
        page = 1;
    } else {
        page++;
    }

    window.location.href = `${url.origin}${url.pathname}?q=${encodeURIComponent(q)}&page=${page}`;
}

/**
 * Navigates to the previous page by decrementing the current page number in the URL query string.
 * @returns {void}
 */
function navigate_backward() {
    let url = new URL(window.location);
    let searchParams = url.searchParams;

    let q = searchParams.get('q');
    let page = parseInt(searchParams.get('page'));

    if (isNaN(page)) {
        page = 0;
    } else if (page > 0) {
        page--;
    }

    window.location.href = `${url.origin}${url.pathname}?q=${encodeURIComponent(q)}&page=${page}`;
}
