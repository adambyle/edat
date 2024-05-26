const elTitle = document.getElementById("title") as HTMLHeadingElement;
const elTitleSpan = elTitle.children[0] as HTMLSpanElement;
const TITLE_MARGIN = 80;

const elPageTitle = document.getElementsByClassName("page-title")[0] as HTMLElement;
let pageTitleShown = false;
let pageTitleTimeout: number;
if (elPageTitle) {
    window.addEventListener("scroll", () => {
        let pageTitleBottom = elPageTitle.getBoundingClientRect().bottom;
        let headerBottom = elTitle.getBoundingClientRect().bottom;
        if (pageTitleBottom < headerBottom && !pageTitleShown) {
            pageTitleShown = true;
            clearTimeout(pageTitleTimeout);
            elTitleSpan.style.opacity = "0.0";
            pageTitleTimeout = setTimeout(() => {
                elTitleSpan.classList.add("page-title-ref");
                elTitleSpan.innerHTML = elPageTitle.innerHTML;
                elTitleSpan.style.opacity = "1.0";
            }, 100);
        }
        if (pageTitleBottom > headerBottom && pageTitleShown) {
            pageTitleShown = false;
            clearTimeout(pageTitleTimeout);
            elTitleSpan.style.opacity = "0.0";
            pageTitleTimeout = setTimeout(() => {
                elTitleSpan.classList.remove("page-title-ref");
                elTitleSpan.innerHTML = "Every Day’s a Thursday";
                elTitleSpan.style.opacity = "1.0";
            }, 100);
        }
    });
}

elTitle.onclick = () => {
    window.scrollTo({
        left: 0,
        top: 0,
        behavior: "smooth",
    });
}

export function titleClick(handler: () => void) {
    elTitle.onclick = handler;
}

let scrolledPast = false;
window.addEventListener("scroll", () => {
    if (elTitle.getBoundingClientRect().top <= -10 && !scrolledPast) {
        document.body.classList.add("scrolled-past");
        scrolledPast = true;
    } else if (window.scrollY < 10 && scrolledPast) {
        scrolledPast = false;
        document.body.classList.remove("scrolled-past");
    }
});
