import * as universal from "./universal.js";
import "./standard.js";

universal.processUtcs();

const elRecentExpand = document.getElementById("recent-expand") as HTMLButtonElement | null;
const elRecentCarousel = document.getElementById("recent-carousel") as HTMLDivElement;
if (elRecentExpand) {
    const unreadSection = elRecentCarousel.querySelector(".section[edat-unread]");
    if (unreadSection) {
        setTimeout(() => {
            elRecentCarousel.scrollBy({
                left: unreadSection.getBoundingClientRect().left - 48,
                behavior: "smooth",
            });
        }, 500);
    }

    const skipButtons: NodeListOf<HTMLButtonElement> =
        document.querySelectorAll("#recent-widget .skip");
    for (const el of skipButtons) {
        el.onclick = () => {
            const unreadMessage = el.previousSibling as HTMLSpanElement;

            fetch(`/read/${el.getAttribute("edat-section")}`, { method: "POST" }).then(() => {
                unreadMessage.innerText = "Marked as read";
            });

            el.remove();
        }
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

const searchInput = document.getElementById("search-input") as HTMLInputElement | null;
if (searchInput) {
    let numberShown = false;

    addEventListener("scroll", () => {
        if (numberShown) {
            return;
        }

        if (searchInput.getBoundingClientRect().bottom < window.innerHeight) {
            numberShown = true;
            let targetNumber = Number.parseInt(
                searchInput.getAttribute("edat_total")!);
            let currentNumber = 0;

            const peak = 0.2;
            const endRate = 0.14;

            setTimeout(() => {
                setInterval(() => {
                    const diff = targetNumber - currentNumber;

                    const k = Math.PI
                        - Math.PI ** (1 - currentNumber / targetNumber)
                        * Math.asin(endRate / peak) ** (currentNumber / targetNumber);
                    const mult = peak * Math.sin(k);

                    currentNumber += diff * (0.00001 + mult);
                    searchInput.placeholder = `Search ${Math.ceil(currentNumber)} words of content`;
                }, 10);
            }, 500);
        }
    });

    searchInput.onkeydown = (ev) => {
        if (ev.key == "Enter") {
            const words = searchInput.value.split(" ").filter(s => s.length > 0);
            const search = words.join(",");
            location.href = `/search/${search}`;
        }
    }
}

// Comment thread handling.
const threads = document.getElementsByClassName("thread") as HTMLCollectionOf<HTMLElement>;
for (const thread of threads) {
    const elBody = thread.querySelector(".body") as HTMLDivElement;
    const elHighlight = elBody.querySelector(".highlight") as HTMLDivElement;

    const scrollAmount = elHighlight.getBoundingClientRect().top
        - elBody.getBoundingClientRect().top
        - 24;
    elBody.scrollTo(0, scrollAmount);
}
