"use strict";
const recordChoices = document.getElementsByClassName("record-choice");
const entries = document.getElementsByClassName("entry");
const elChooseEntries = document.getElementById("choose-entries");
const elConfigure = document.getElementById("configure");
let readEntries = [];
let scrolled = false;
for (const choice of recordChoices) {
    choice.onclick = () => {
        for (const otherChoice of recordChoices) {
            otherChoice.classList.remove("selected");
        }
        choice.classList.add("selected");
        if (choice.id == "custom-record") {
            elChooseEntries.style.opacity = "1.0";
            elChooseEntries.style.height = "unset";
            elChooseEntries.style.height = `${elChooseEntries.getBoundingClientRect().height + 24}px`;
            elChooseEntries.style.marginBottom = "24px";
            elChooseEntries.style.padding = "12px 24px";
            setTimeout(() => {
                window.scrollBy({
                    behavior: "smooth",
                    top: elChooseEntries.getBoundingClientRect().top - 48,
                });
            }, 300);
        }
        else {
            elChooseEntries.style.opacity = "0.0";
            elChooseEntries.style.height = "0";
            elChooseEntries.style.marginBottom = "0";
            elChooseEntries.style.padding = "0 24px";
            if (!scrolled) {
                setTimeout(() => {
                    scrolled = true;
                    window.scrollBy({
                        behavior: "smooth",
                        top: elConfigure.getBoundingClientRect().top - 48,
                    });
                }, 300);
            }
        }
        elConfigure.style.display = "block";
        setTimeout(() => elConfigure.style.opacity = "1.0", 50);
    };
}
for (const entry of entries) {
    const entryId = entry.getAttribute("edat-entry");
    entry.onclick = () => {
        if (entry.classList.contains("selected")) {
            entry.classList.remove("selected");
            readEntries = readEntries.filter(e => e != entryId);
        }
        else {
            entry.classList.add("selected");
            readEntries.push(entryId);
        }
    };
}
