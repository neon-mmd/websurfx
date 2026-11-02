/**
 * This function provides the ability for the button to toggle the dropdown error-box
 * in the search page.
 */
function toggleErrorBox() {
    document.querySelector('.dropdown_error_box')?.classList.toggle('show')
}

if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        document.querySelectorAll('.error_box_toggle_button').forEach((btn) => {
            btn.addEventListener('click', toggleErrorBox)
        })
    })
} else {
    document.querySelectorAll('.error_box_toggle_button').forEach((btn) => {
        btn.addEventListener('click', toggleErrorBox)
    })
}
