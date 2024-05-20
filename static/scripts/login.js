"use strict";
const nameInput = document.getElementById("name-input");
const codeInput = document.getElementById("code-input");
const errorMsg = document.getElementById("error-msg");
document.body.onkeydown = ev => {
    errorMsg.style.display = "none";
    if (ev.key == "Enter") {
        login();
    }
};
function login() {
    fetch(`/login/${nameInput.value}/${codeInput.value}`, { method: "POST" }).then(res => {
        console.log(res.status);
        if (res.status == 200) {
            res.text().then(text => {
                document.cookie = `edat_user=${text}`;
                window.location.reload();
            });
        }
        else {
            errorMsg.style.display = "block";
        }
    });
}
