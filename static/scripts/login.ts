import "./universal.js";

const loginButton = document.getElementById("login-button") as HTMLButtonElement;
const nameInput = document.getElementById("name-input") as HTMLInputElement;
const codeInput = document.getElementById("code-input") as HTMLInputElement;
const errorMsg = document.getElementById("error-msg")!;

loginButton.onclick = login;

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
                const expirationDate = new Date(Date.now() + 100 * 1000 * 60 * 60 * 24 * 365);
                window.location.reload();
            });
        } else {
            errorMsg.style.display = "block";
        }
    });
}
