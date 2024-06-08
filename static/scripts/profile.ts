import * as universal from "./universal.js"
import { includeWidgets, setClickExtras } from "./widgets.js"
universal.processUtcs();

setClickExtras(() => {
    fetch("/widgets", {
        method: "POST",
        body: JSON.stringify(includeWidgets),
        headers: {
            "Content-Type": "application/json",
        },
    });
});

let widgetsShown = false;
const widgetExpandButton = document.getElementById("homepage-expand") as HTMLButtonElement;
const elWidgets = document.getElementById("widgets") as HTMLDivElement;
const elHomepage = document.getElementById("homepage") as HTMLDivElement;
widgetExpandButton.onclick = () => {
    if (widgetsShown) {
        elWidgets.style.display = "none";
        setTimeout(() => {
            elWidgets.style.opacity = "0.0";
        }, 50);
        widgetsShown = false;
        widgetExpandButton.innerText = "Expand";

        setTimeout(() => {
            window.scrollBy({
                top: elHomepage.getBoundingClientRect().top - 48,
                behavior: "smooth",
            });
        }, 100);

    } else {
        elWidgets.style.display = "block";
        setTimeout(() => {
            elWidgets.style.opacity = "1.0";
        }, 50);
        widgetsShown = true;
        widgetExpandButton.innerText = "Collapse";
    }
}

let historyExpanded = false;
const historyExpandButton = document.getElementById("history-expand") as HTMLButtonElement;
const elHistoryRest = document.getElementById("history-rest") as HTMLDivElement;
const elHistory = document.getElementById("history") as HTMLDivElement;
historyExpandButton.onclick = () => {
    if (historyExpanded) {
        elHistoryRest.style.display = "none";
        setTimeout(() => {
            elHistoryRest.style.opacity = "0.0";
        }, 50);
        historyExpanded = false;
        historyExpandButton.innerText = "Show more";

        setTimeout(() => {
            window.scrollBy({
                top: elHistory.getBoundingClientRect().top - 48,
                behavior: "smooth",
            });
        }, 100);
    } else {
        elHistoryRest.style.display = "block";
        setTimeout(() => {
            elHistoryRest.style.opacity = "1.0";
        }, 50);
        historyExpanded = true;
        historyExpandButton.innerText = "Show less";
    }
};

const homeButton = document.getElementById("home") as HTMLButtonElement;
homeButton.onclick = () => {
    if (document.referrer == "") {
        location.replace("/");
    } else {
        location.replace(document.referrer);
    }
}
