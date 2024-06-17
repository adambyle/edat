export let includeWidgets: string[] = [];

let clickExtras = () => {};

export function setClickExtras(fn: () => void) {
    clickExtras = fn;
}

const widgets = document.getElementsByClassName("widget") as HTMLCollectionOf<HTMLDivElement>;
for (const widget of widgets) {
    const button = widget.children[1] as HTMLButtonElement;
    const span = widget.children[0] as HTMLSpanElement;
    if (button.classList.contains("selected")) {
        includeWidgets.push(button.id);
    }
    
    button.onclick = () => {
        if (button.classList.contains("selected")) {
            button.classList.remove("selected");
            includeWidgets = includeWidgets.filter(w => w != button.id);
            span.style.opacity = "0.0";

            for (let i = 0; i < includeWidgets.length; i++) {
                const changeSpan = document
                    .getElementById(includeWidgets[i])!
                    .parentElement!
                    .children[0] as HTMLSpanElement;
                changeSpan.innerText = `#${i + 1}`;
            }
        } else {
            button.classList.add("selected");
            includeWidgets.push(button.id);
            span.innerText = `#${includeWidgets.length}`;
            span.style.opacity = "1.0";
        }
        clickExtras();
    }
}

const widgetSelectAllButtons = document
    .getElementsByClassName("widget-select-all") as HTMLCollectionOf<HTMLButtonElement>;

for (const button of widgetSelectAllButtons) {
    button.onclick = () => {
        for (const widget of widgets) {
            const button = widget.children[1] as HTMLButtonElement;
            if (!button.classList.contains("selected")) {
                button.click();
            }
        }
    }
}
