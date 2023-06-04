/* 
    UI: Method for selecting all search engines
    
    This function is called when user click on
    `select all` and `deselect all` toogle element.
    
    - If select all is pressed toggle all options and insert all value
      joined with comma to input which is saved in cookie.
      then changes text to `deselect all`.
      eg value: 'ddg,searx,'

    - If deselect all is pressed uncheck all options and insert empty
      value to input which is saved in cookie.
*/
function selectAllHandler(elem) {
  let span = elem.parentElement.querySelector("span");
  let mainInput = document.getElementsByName("searchEng")[0];
  let prevStateSelectAll = span.innerText == "Select all" ? true : false;
  document.querySelectorAll(".searchEng-elem").forEach((engine) => {
    if (prevStateSelectAll) {
      engine.querySelector('input[type="checkbox"]').checked = true;
    } else {
      engine.querySelector('input[type="checkbox"]').checked = false;
    }
  });
  if (prevStateSelectAll) {
    let getValues = () => {
      let value = "";
      document
        .querySelectorAll('[data-isCheckbox]:not([data-value="all"])')
        .forEach((elem) => {
          value += elem.getAttribute("data-value") + ",";
        });
      return value;
    };
    mainInput.value = getValues();
  } else {
    mainInput.value = "";
  }
  span.innerText = prevStateSelectAll ? "Deselect all" : "Select all";
}

/* 
    UI: Filter settings as per category
    
    There are two elements one is `category` and `detail`.
    Category contains `data-id` when a user click it
    we need to show detail element having 
    `data-detailId` whose value is same as `data-id`.
    
    When a user clicks on a category on sidebar, view
    settings containing `data-id` of sidebar element's `data-detailId`
    and hide other settings

    - if `all` is clicked then show all settings.
*/
document.querySelectorAll(".settings-sidebar .set-name").forEach((filter) => {
  let target = filter.getAttribute("data-detailId");
  filter.addEventListener("click", () => {
    try {
      document.querySelector(".set-name.active").classList.remove("active");
    } catch (e) {}
    filter.classList.add("active");
    if (target == "all") {
      document.querySelectorAll(".set-item").forEach((elem) => {
        elem.style.display = "block";
      });
      return;
    }
    document
      .querySelectorAll('.set-item[data-id="' + target + '"]')
      .forEach((elem) => {
        elem.style.display = "block";
      });
    document
      .querySelectorAll('.set-item:not([data-id="' + target + '"])')
      .forEach((elem) => {
        elem.style.display = "none";
      });
  });
});


/*
    This function is called when a user click on submit button
    it validates all user inputs and saves to a cookie.
    
    - if input having `required` attr and is empty it generates a
      error text and insert above the setting element
    
    - else store setttings to a cookie.
*/
function submitSettings() {
  let form = document.settings;
  let stopProceeding = false;
  document.querySelectorAll(".errTxt").forEach((e) => e.remove());
  for (let i = 0; i < form.elements.length; i++) {
    let input = form.elements[i];
    if (input.value == "" && input.hasAttribute("required")) {
      stopProceeding = true;
      let elem = input.parentElement.querySelector("[takeInput]");
      let errTxt = document.createElement("p");
      errTxt.classList.add("errTxt");
      errTxt.innerText = "This setting can't be empty!!";
      elem.classList.add("invalid");
      elem.parentElement.insertBefore(errTxt, elem);
      let sidebarElement = input.closest(".set-item").getAttribute("data-id");
      document
        .querySelector(
          `.settings-sidebar .set-name[data-detailId="${sidebarElement}`
        )
        .click();
      stopProceeding = true;
    }
  }
  if (!stopProceeding) {
    var expiration_date = new Date();
    expiration_date.setFullYear(expiration_date.getFullYear() + 1);
    let formData = new FormData(document.querySelector("form"));
    for (var [key, value] of formData.entries()) {
      document.cookie = `${key}=${value}; expires=${expiration_date.toUTCString()}`;
    }
  } else {
    return false;
  }
  // On settings saved successfully
  alert("Settings saved succssfully!");
  window.location.reload();
}


/*
    This function will be called on page ready.
    it iterates over saved cookies and if cookie name is 
    in the list of valid cookies then load it.

    - if cookie is searchEng(type=toggle/checkbox) we deselect 
      all checkboxes and select those which are stored in cookie.

    - if cookie is of type `select` we deselect default selected
      option and then select option which is stored in cookie.
*/
function loadUserSettings() {
  let inputs = ["searchEng", "theme", "color-sch"];
  var keyValuePairs = document.cookie.split(";");
  for (var i = 0; i < keyValuePairs.length; i++) {
    var name = keyValuePairs[i].substring(0, keyValuePairs[i].indexOf("="));
    var value = keyValuePairs[i].substring(keyValuePairs[i].indexOf("=") + 1);
    name = name.trim();
    if (!inputs.includes(name)) {
      continue;
    }
    let input = document.getElementsByName(name)[0];
    input.value = value;
    if (name == "searchEng") {
      // Unload all checked engines
      document
        .querySelectorAll(".searchEng-elem input[type=checkbox]")
        .forEach((e) => {
          e.checked = false;
        });
      value = value.replace(" ", "");
      value.split(",").forEach((val) => {
        if (!val) {
          return;
        }
        document
          .querySelector(`[data-isCheckbox][data-value="${val}"]`)
          .parentElement.querySelector("input").checked = true;
      });
    } else {
      // Unload all selected options
      document
        .querySelector(
          `[data-input="${name}"] .options span[data-value="${value}"]`
        )
        .removeAttribute("selected");
      singleSelectClickHandler(
        document.querySelector(`.options span[data-value="${value}"]`)
      );
    }
  }
}

// Code where settings are loaded from cookie.
loadUserSettings();
