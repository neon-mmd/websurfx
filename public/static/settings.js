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

let selectElements = document.querySelectorAll('.custom-select');
Array.from(selectElements).forEach(element => {
    element.childNodes[0].nodeValue = (element.hasAttribute('data-multiple') ? element.getAttribute('data-placeholder') : element.getAttribute('data-default'));
    element.addEventListener('click', (e) => {if (e.target === element) toggleSelectOptions(element)});
    element.addEventListener('focusout', (e) => {if (e.target === element) toggleSelectOptions(element, 'close')});
});

function removeSelectOption(elem, optionId) {
    let option = document.querySelector('#'+optionId);
    let selectDiv = option.closest('.custom-select');
    let input = document.querySelector(`[name="${selectDiv.getAttribute('data-input')}"]`);
    elem.parentElement.remove();
    option.removeAttribute('selected');
    option.querySelector('svg').remove();
    input.value = input.value.replace(option.getAttribute('data-value') ? option.getAttribute('data-value') + "," : '', '');
}

function multiSelectClickHandler(elem) {
    let selectDiv = elem.closest('.custom-select');
    let input = document.querySelector(`[name="${selectDiv.getAttribute('data-input')}"]`);
    if (!elem.hasAttribute('selected')) {
        document.querySelector('#'+elem.closest(".custom-select").getAttribute("data-showDivId")).innerHTML += 
        `<span class='selected-multiple-option' id='${elem.getAttribute('id')}-selected'>${elem.innerText} &ensp; 
          <span onclick="removeSelectOption(this, '${elem.getAttribute('id')}')">
            <svg fill="currentColor" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 384 512"><path d="M376.6 84.5c11.3-13.6 9.5-33.8-4.1-45.1s-33.8-9.5-45.1 4.1L192 206 56.6 43.5C45.3 29.9 25.1 28.1 11.5 39.4S-3.9 70.9 7.4 84.5L150.3 256 7.4 427.5c-11.3 13.6-9.5 33.8 4.1 45.1s33.8 9.5 45.1-4.1L192 306 327.4 468.5c11.3 13.6 31.5 15.4 45.1 4.1s15.4-31.5 4.1-45.1L233.7 256 376.6 84.5z"/>
            </svg>
          </span>
        </span>`;
        elem.setAttribute('selected', '');
        elem.innerHTML = `${elem.innerText}
          <svg fill="currentColor" height="1.5rem" width="1.5rem" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 490 490" xml:space="preserve">
          <g id="SVGRepo_iconCarrier"> <polygon points="452.253,28.326 197.831,394.674 29.044,256.875 0,292.469 207.253,461.674 490,54.528 "></polygon> </g></svg>`

        // Code where value in inserted into input
        input.value += elem.getAttribute('data-value') ? elem.getAttribute('data-value') + "," : '';
    } else {
        // similar to removeSelectOption method
        document.querySelector('#'+elem.getAttribute('id')+'-selected').remove();
        elem.removeAttribute('selected');
        elem.querySelector('svg').remove();

        input.value = input.value.replace(elem.getAttribute('data-value') ? elem.getAttribute('data-value') + "," : '', '');
    }
    
}

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
        elem.innerHTML = `${elem.innerText}
          <svg fill="currentColor" height="1.5rem" width="1.5rem" xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 490 490" xml:space="preserve">
          <g id="SVGRepo_iconCarrier"> <polygon points="452.253,28.326 197.831,394.674 29.044,256.875 0,292.469 207.253,461.674 490,54.528 "></polygon> </g></svg>`
          // Code where value is inserted to input
          input.value = elem.getAttribute('data-value') ? elem.getAttribute('data-value') : '';
          selectDiv.childNodes[0].nodeValue = elem.innerText;
    } else {
        elem.removeAttribute('selected');
        elem.querySelector('svg').remove();
        selectDiv.childNodes[0].nodeValue = selectDiv.getAttribute('data-default');

        input.value = "";
    }
    selectDiv.blur();
}

let multiSelectOptions = document.querySelectorAll('.custom-select[data-multiple="1"]>.options span');
for (let i = 0; i < multiSelectOptions.length; i++) {
    multiSelectOptions[i].addEventListener('click', () => {multiSelectClickHandler(multiSelectOptions[i])});
    multiSelectOptions[i].setAttribute('id', 'option-'+i.toString());
}

let singleSelectOptions = document.querySelectorAll('.custom-select:not([data-multiple="1"])>.options span');
for (let i = 0; i < singleSelectOptions.length; i++) {
    singleSelectOptions[i].addEventListener('click', () => {singleSelectClickHandler(singleSelectOptions[i])});
    singleSelectOptions[i].setAttribute('id', 'option-'+i.toString());
}

function selectAllHandler(elem) {
    let options = elem.parentElement.querySelectorAll('.custom-select[data-multiple="1"]>.options span');
    Array.from(options).forEach((option) => {
        try {
            let selectedShownElem = document.getElementById(option.getAttribute('id')+'-selected');
            removeSelectOption(selectedShownElem.querySelector('span'), option.getAttribute('id'))
        } catch (err) {}
        if (elem.innerText == 'select all') { option.click() };
    });
    elem.innerText = elem.innerText == 'select all' ? 'deselect all' : 'select all'
}


// On settings form submit
function submitSettings() {
    let form = document.settings;
    document.querySelectorAll('.errTxt').forEach(e => e.remove());
    for(let i = 0; i < form.elements.length; i++) {
        let input = form.elements[i];
        if (input.value == "" && input.hasAttribute('required')) {
            let elem = input.parentElement.querySelector('[takeInput]')
            let errTxt = document.createElement('p')
            errTxt.classList.add('errTxt')
            errTxt.innerText = 'This setting can\'t be empty!!'
            elem.classList.add('invalid');
            elem.parentElement.insertBefore(errTxt, elem);
        }
    }
    return false;
}

document.querySelectorAll('.settings-sidebar .set-name').forEach(filter => {
    let target = filter.getAttribute('data-detailId');
    filter.addEventListener('click', () => {
        try {document.querySelector('.set-name.active').classList.remove('active');} catch(e){}
        filter.classList.add('active');
        if (target == 'all') {
            document.querySelectorAll('.set-item').forEach((elem) => {
                elem.style.display = 'block';
            })
            return;
        }
        document.querySelectorAll('.set-item[data-id="'+target+'"]').forEach((elem) => {
            elem.style.display = 'block'
        })
        document.querySelectorAll('.set-item:not([data-id="'+target+'"])').forEach((elem) => {
            elem.style.display = 'none'
        })
    })
})

function fadeOut(element) {
    var op = 1;  // initial opacity
    var timer = setInterval(function () {
        if (op <= 0.1){
            clearInterval(timer);
            element.style.display = 'none';
            element.classList.add('fade');
        }
        element.style.opacity = op;
        element.style.filter = 'alpha(opacity=' + op * 100 + ")";
        op -= op * 0.1;
    }, 50);
}

function fadeIn(element) {
    var op = 0.1;  // initial opacity
    element.style.display = 'block';
    var timer = setInterval(function () {
        if (op >= 1){
            clearInterval(timer);
        }
        element.style.opacity = op;
        element.style.filter = 'alpha(opacity=' + op * 100 + ")";
        op += op * 0.1;
    }, 10);
}
