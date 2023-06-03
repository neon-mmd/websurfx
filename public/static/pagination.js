function navigate_forward() {
    const url = new URL(window.location.href);
    const searchParams = url.searchParams;

    let q = searchParams.get('q');
    let page = searchParams.get('page');

    if (page === null) {
        page = 2;
    } else {
        page = parseInt(page) + 1;
    }

    url.searchParams.set('q', q);
    url.searchParams.set('page', page);
    window.location.href = url.toString();
}

function navigate_backward() {
    const url = new URL(window.location.href);
    const searchParams = url.searchParams;

    let q = searchParams.get('q');
    let page = searchParams.get('page');

    if (page !== null && page > 1) {
        page = parseInt(page) - 1;
        url.searchParams.set('q', q);
        url.searchParams.set('page', page);
        window.location.href = url.toString();
    }
}
