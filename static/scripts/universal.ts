// Theme handling.
if (!localStorage.edatTheme) {
    changeThemeSetting("system");
} else if (!document.cookie.includes("edat_theme")) {
    // Force reset theme.
    const theme = localStorage.edatTheme;
    document.cookie = `edat_theme=${theme}; Max-Age=31536000`;
    localStorage.removeItem("edatTheme");
    changeTheme(theme);
}

export function changeThemeSetting(themeSetting: string) {
    if (themeSetting == localStorage.edatThemeSetting) {
        return;
    }
    localStorage.edatThemeSetting = themeSetting;

    if (themeSetting == "system") {
        if (matchMedia("(prefers-color-scheme: dark)").matches) {
            changeTheme("dark");
        } else {
            changeTheme("light");
        }
    } else {
        changeTheme(themeSetting);
    }
}

export function changeTheme(theme: string) {
    if (theme == localStorage.edatTheme) {
        return;
    }
    localStorage.edatTheme = theme;
    document.cookie = `edat_theme=${theme}; Max-Age=31536000`;

    if (theme == "dark") {
        document.body.classList.add("dark-theme");
    } else {
        document.body.classList.remove("dark-theme");
    }
}

export function nowString() {
    const now = new Date(Date.now());
    return `${now.getFullYear()}-${now.getMonth() + 1}-${now.getDate()}`;
}

matchMedia("(prefers-color-scheme: dark)").addEventListener("change", ev => {
    if (localStorage.edatThemeSetting == "system") {
        changeTheme(ev.matches ? "dark" : "light");
    }
});

const MONTHS = [
    "Jan",
    "Feb",
    "Mar",
    "Apr",
    "May",
    "Jun",
    "Jul",
    "Aug",
    "Sep",
    "Oct",
    "Nov",
    "Dec"
];

export function processUtcs() {
    const utcs = document.querySelectorAll("utc:not(.processed)") as NodeListOf<HTMLElement>;
    for (const el of utcs) {
        el.classList.add("processed");
        let utcSeconds = Number.parseInt(el.innerText);
        const date = new Date(utcSeconds * 1000);
        const now = new Date(Date.now());
        if (now.getFullYear() == date.getFullYear()) {
            el.innerHTML = `${MONTHS[date.getMonth()]} ${date.getDate()}`;
        } else {
            el.innerHTML = `${MONTHS[date.getMonth()]} ${date.getDate()}, ${date.getFullYear()}`;
        }
        if (el.classList.contains("cap")) {
            el.innerHTML = el.innerHTML.toUpperCase();
        }
        el.style.opacity = "1.0";
    }
}
