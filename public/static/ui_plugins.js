/*  
    Select plugin
    Note: Only for single select option
    Usage example: 
    ```
    <input name="theme" disabled style="display: none;" required></input>
    <div class="custom-select" tabindex="0" data-input="theme" data-default="Select" takeInput>
        Svg Icon here
        <div class="options">
            <span data-value="simple">Simple</span>
        </div>
    </div>
    ```
*/

let svg_tick = `
  <svg fill="currentColor" height="1.5rem" width="1.5rem" viewBox="0 0 490 490" xml:space="preserve">
    <g id="SVGRepo_iconCarrier">
      <polygon points="452.253,28.326 197.831,394.674 29.044,256.875 0,292.469 207.253,461.674 490,54.528 "></polygon>
    </g>
  </svg>`;

function toggleSelectOptions(elem, state) {
    elem.classList.remove("invalid");
    try { elem.parentElement.querySelector('.errTxt').remove();} catch (error) {}
    let options = elem.querySelector('.options');
    const pos = elem.getBoundingClientRect();
    const windowWidth = document.getElementsByTagName("body")[0].clientHeight;
    if(pos.y + 250 > windowWidth) {
        options.style.bottom = '40px';
    } else { options.style.bottom = null }
    options.style.display = state != 'close' ? getComputedStyle(options).display == 'none' ? 'block': 'none' : 'none';
}

let selectElements = document.querySelectorAll('.custom-select').forEach(element => {
    let value = element.getAttribute('data-default')
    element.childNodes[0].nodeValue = value;
    element.querySelector(`.options span[data-value="${value.toLowerCase()}"]`).
      innerHTML = `${value} ${svg_tick}`;
    element.addEventListener('click', (e) => {if (e.target === element) toggleSelectOptions(element)});
    element.addEventListener('focusout', (e) => {if (e.target === element) toggleSelectOptions(element, 'close')});
});

function singleSelectClickHandler(elem) {
    let selectDiv = elem.closest('.custom-select');
    let selectedOption = selectDiv.querySelector('span[selected]');
    let input = document.querySelector(`[name="${selectDiv.getAttribute('data-input')}"]`);
    if (!elem.hasAttribute('selected')) {
        if (selectedOption != null) {
            selectedOption.removeAttribute('selected');
            selectedOption.querySelector('svg').remove();
        }
        elem.setAttribute('selected', '');
        elem.innerHTML = `${elem.innerText} ${svg_tick}`
        // Code where value is inserted to input
        input.value = elem.getAttribute('data-value') ? elem.getAttribute('data-value') : '';
        selectDiv.childNodes[0].nodeValue = elem.innerText;
    } else {
        elem.removeAttribute('selected');
        elem.querySelector('svg').remove();
        selectDiv.childNodes[0].nodeValue = selectDiv.getAttribute('data-defaultIfNone');

        input.value = "";
    }
    selectDiv.blur();
}

let singleSelectOptions = document.querySelectorAll('.custom-select:not([data-multiple="1"])>.options span');
for (let i = 0; i < singleSelectOptions.length; i++) {
    singleSelectOptions[i].addEventListener('click', () => {singleSelectClickHandler(singleSelectOptions[i])});
    singleSelectOptions[i].setAttribute('id', 'option-'+i.toString());
}


/*
    Toggle switch plugin
    Usage example:
    ```
    <input type="hidden" name="searchEng" disabled>
    <div class="searchEng">
        <div class="searchEng-elem">
            <input type="checkbox" id="toggle"/>
            <div data-isCheckbox data-input="searchEng" data-value="ddg">
                <label for="toggle"></label>
            </div>
            <span>Duck duck go</span>
        </div>
        <div class="searchEng-elem">
            <input type="checkbox" id="toggle1"/>
            <div data-isCheckbox data-input="searchEng" data-value="searx">
                <label for="toggle1"></label>
            </div>
            <span>Searx</span>
        </div>
    </div>
    ```
*/

document.querySelectorAll('[data-isCheckbox]:not([data-value="all"]) label').forEach(checkBoxLabel => { 
    checkBoxLabel.addEventListener('click', () => {
        let checkBox = checkBoxLabel.parentElement;
        let helperInput = checkBox.parentElement.querySelector('input[type="checkbox"]');
        let mainInput = document.getElementsByName(checkBox.getAttribute('data-input'))[0];
        if (helperInput.checked == true) {
            mainInput.value = mainInput.value.replace(checkBox.getAttribute('data-value') + ',', '');
        } else {
            mainInput.value += checkBox.getAttribute('data-value') + ',';
        }
    });
})
