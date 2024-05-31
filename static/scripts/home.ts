import * as universal from "./universal.js";
import "./standard.js";

universal.processUtcs();

const elRecentExpand = document.getElementById("recent-expand") as HTMLButtonElement | null;
const elRecentCarousel = document.getElementById("recent-carousel") as HTMLDivElement;
if (elRecentExpand) {
    const unreadSection = elRecentCarousel.querySelector(".section[edat-unread]");
    if (unreadSection) {
        elRecentCarousel.scrollBy({
            left: unreadSection.getBoundingClientRect().left - 48,
            behavior: "smooth",
        });
    }

    const concise = document.getElementsByClassName("concise") as HTMLCollectionOf<HTMLElement>;
    const detailed = document.getElementsByClassName("detailed") as HTMLCollectionOf<HTMLElement>;
    let expandTimeout: number;
    elRecentExpand.onclick = () => {
        elRecentExpand.style.opacity = "0";
        if (elRecentCarousel.classList.contains("show-concise")) {
            fetch("/preferences", {
                method: "POST",
                body: JSON.stringify({
                    expand_recents: "true",
                }),
                headers: {
                    "Content-Type": "application/json",
                },
            });
            clearTimeout(expandTimeout);
            for (const elem of detailed) {
                elem.style.display = "";
            }
            setTimeout(() => {
                elRecentCarousel.classList.remove("show-concise");
                elRecentCarousel.classList.add("show-detailed");
            }, 10);

            expandTimeout = setTimeout(() => {
                for (const elem of concise) {
                    elem.style.display = "none";
                }
                elRecentExpand.style.opacity = "1";
            }, 200);
        } else {
            fetch("/preferences", {
                method: "POST",
                body: JSON.stringify({
                    expand_recents: "false",
                }),
                headers: {
                    "Content-Type": "application/json",
                },
            });
            clearTimeout(expandTimeout);
            for (const elem of concise) {
                elem.style.display = "";
            }
            setTimeout(() => {
                elRecentCarousel.classList.remove("show-detailed");
                elRecentCarousel.classList.add("show-concise");
            }, 10);
            expandTimeout = setTimeout(() => {
                for (const elem of detailed) {
                    elem.style.display = "none";
                }
                elRecentExpand.style.opacity = "1";
            }, 200);
        }
    }
}
