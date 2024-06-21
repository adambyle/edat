import "./universal.js";
import "./standard.js";
import * as standard from "./standard.js";

// Jump to the specified position if one exists.
const elHere = document.querySelector(".here") as HTMLElement;
if (elHere) {
    let scrollOffset: number;
    if (elHere.classList.contains("here-section")) {
        scrollOffset = 240;
    } else {
        scrollOffset = window.innerHeight;
    }

    setTimeout(() => {
        window.scrollBy({
            top: elHere.getBoundingClientRect().top - scrollOffset,
            behavior: "smooth",
        });
    }, 500);
}

// Force the page to double-load next time, to update reading records.
setTimeout(() => {
    localStorage.setItem("edat_force_reload", "true");
}, 10000);

// Section navigation.
const topSections = document
    .getElementsByClassName("topsection") as HTMLCollectionOf<HTMLElement>;
for (const section of topSections) {
    if (!section.classList.contains("missing")) {
        section.onclick = () => {
            const sectionId = section.getAttribute("edat_section");
            const sectionElement = document.querySelector(`.section[edat_section="${sectionId}"]`)!;
            standard.closeHeader();
            scrollBy({
                top: sectionElement.getBoundingClientRect().top - 240,
                behavior: "smooth",
            });
        }
    }
}

function updateNav() {
    for (const section of topSections) {
        const sectionId = section.getAttribute("edat_section");
        const status = section.querySelector(".unread");
        if (!status) {
            continue;
        }
        if (sectionId && sectionId == readingNow.toString()) {
            status.innerHTML = "Reading now";
            section.classList.add("reading-now");
        } else {
            section.classList.remove("reading-now");
            if (section.classList.contains("unread")) {
                status.innerHTML = "Unread";
            } else {
                status.innerHTML = "";
            }
        }
    }
}

standard.setOnHeaderOpen(() => {
    const elReadingNow = document.querySelector(".reading-now");
    if (elReadingNow) {
        elReadingNow.scrollIntoView({ behavior: "smooth", block: "center" });
    }
});

interface Progress {
    element: HTMLElement,
    id: number,
    progress: number,
    finished: boolean,
    onScreenTime: number,
    startOnScreenTime: number,
    endOnScreenTime: number,
    lines: NodeListOf<Element>,
}

// Track progress through individual sections.
// Start the engine when the user has scrolled once.
window.addEventListener("scroll", startProgressEngine);

let progressEngineStarted = false;
let sections: Progress[] = [];
let readingNow = -1;
function startProgressEngine() {
    if (progressEngineStarted) {
        return;
    }
    progressEngineStarted = true;

    for (const section of document.querySelectorAll(".section")) {
        let section_id = section.getAttribute("edat_section");
        if (section_id) {
            const lines = section.querySelectorAll(".textline");
            sections.push({
                element: section as HTMLElement,
                id: Number.parseInt(section_id),
                progress: 0,
                finished: false,
                onScreenTime: 0,
                startOnScreenTime: 0,
                endOnScreenTime: 0,
                lines,
            });
        }
    }

    const PROGRESS_TIMER = 20;
    const FINISH_TIMER = 10;

    setInterval(() => {
        for (const section of sections) {
            // Section must be onscreen for at least a minute.
            const rect = section.element.getBoundingClientRect();
            if (rect.bottom > 0 && rect.top < window.innerHeight) {
                section.onScreenTime += 0.1;
                if (readingNow != section.id) {
                    readingNow = section.id;
                    updateNav();
                }
            }

            // Determine scanline position.
            const proportion = 1 - (
                (section.element.getBoundingClientRect().bottom - window.innerHeight)
                / section.element.clientHeight);
            const scanlineY = window.innerHeight * proportion;

            for (const line of section.lines) {
                // Check that the line is onscreen.
                const rect = line.getBoundingClientRect();
                if (rect.bottom < 0 || rect.top > window.innerHeight) {
                    continue;
                }

                // Increment start/end times.
                if (line == section.lines.item(0)) {
                    section.startOnScreenTime += 0.1;
                }
                if (line == section.lines.item(section.lines.length - 1)) {
                    section.endOnScreenTime += 0.1;
                }
                if (section.endOnScreenTime >= FINISH_TIMER
                    && section.startOnScreenTime >= FINISH_TIMER
                    && !section.finished
                ) {
                    section.finished = true;

                    const nav = document
                        .querySelector(`.topsection[edat_section="${section.id}"]`)!;
                    nav.classList.remove("unread");
                }

                // Check that we've progressed further.
                const line_no = Number.parseInt(line.getAttribute("edat_line")!);
                if (line_no <= section.progress) {
                    continue;
                }

                // Log progress.
                if (section.onScreenTime >= PROGRESS_TIMER
                    && rect.bottom >= scanlineY
                    && !section.finished
                ) {
                    section.progress = line_no;

                    const nav = document
                        .querySelector(`.topsection[edat_section="${section.id}"]`)!;
                    nav.classList.remove("unread");
                }

                // If this is the last one, section finished.
                if (line == section.lines.item(section.lines.length - 1)
                    && section.onScreenTime >= PROGRESS_TIMER
                    && !section.finished
                ) {
                    section.finished = true;

                    const nav = document
                        .querySelector(`.topsection[edat_section="${section.id}"]`)!;
                    nav.classList.remove("unread");
                }
            }
        }
    }, 100);
}

document.addEventListener("visibilitychange", () => {
    for (const section of sections) {
        if (section.finished) {
            navigator.sendBeacon(`/read/${section.id}`);
        } else if (section.progress > 0) {
            navigator.sendBeacon(`/read/${section.id}?progress=${section.progress}`);
        }
    }
});

// Image opening.
const imgs = document.querySelectorAll(".img") as NodeListOf<HTMLElement>;
for (const img of imgs) {
    img.addEventListener("click", () => {
        const open = img.querySelector(".open")! as HTMLElement;
        open.style.display = "none";
        const image = img.querySelector("img")!;
        image.style.display = "block";
        setTimeout(() => {
            image.style.opacity = "1";
        }, 10);
    });
}
