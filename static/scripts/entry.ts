import "./universal.js";
import "./standard.js";

// Jump to the specified position if one exists.
const elHere = document.querySelector(".here") as HTMLElement;
if (elHere) {
    setTimeout(() => {
        window.scrollBy({
            top: elHere.getBoundingClientRect().top - window.innerHeight,
            behavior: "smooth",
        });
    }, 500);
}

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
                    console.log("Start on screen");
                    section.startOnScreenTime += 0.1;
                }
                if (line == section.lines.item(section.lines.length - 1)) {
                    console.log("End on screen");
                    section.endOnScreenTime += 0.1;
                }
                if (section.endOnScreenTime >= FINISH_TIMER
                    && section.startOnScreenTime >= FINISH_TIMER
                    && !section.finished
                ) {
                    console.log("FINISHED");
                    section.finished = true;
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
                    console.log("Progress ", line_no);
                }

                // If this is the last one, section finished.
                if (line == section.lines.item(section.lines.length - 1)
                    && section.onScreenTime >= PROGRESS_TIMER
                    && !section.finished
                ) {
                    section.finished = true;
                    console.log("FINISHED");
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
