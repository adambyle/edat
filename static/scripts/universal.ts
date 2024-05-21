// Theme handling.
if (!localStorage.edatTheme) {
    changeThemeSetting("system");
}

function changeThemeSetting(themeSetting: string) {
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

function changeTheme(theme: string) {
    if (theme == localStorage.edatTheme) {
        return;
    }
    localStorage.edatTheme = theme;
    document.cookie = `edat_theme=${theme}`;
    
    if (theme == "dark") {
        document.body.classList.add("dark-theme");
    } else {
        document.body.classList.remove("dark-theme");
    }
}

matchMedia("(prefers-color-scheme: dark)").addEventListener("change", ev => {
    if (localStorage.edatThemeSetting == "system") {
        changeTheme(ev.matches ? "dark" : "light");
    }
});
