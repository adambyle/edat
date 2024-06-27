import "./universal.js";
import * as universal from "./universal.js";
import "./standard.js";
import * as standard from "./standard.js";

// Comment handling.
let openComment: HTMLElement | null = null;

let touchTime = 0;

let editUuid: string | null = null;

const textlines = document.getElementsByClassName("textline") as HTMLCollectionOf<HTMLElement>;

for (const textline of textlines) {
    textline.onclick = ev => {
        const lastClick = touchTime;
        touchTime = Date.now();
        if (touchTime - lastClick > 600) {
            return;
        }
        ev.preventDefault();
        showComments(textline);
    };
}

function showComments(lineElement: HTMLElement) {
    if (lineElement == openComment) {
        hideComments();
        return;
    }

    if (openComment) {
        hideComments();
    }

    openComment = lineElement;

    const elThread = document.createElement("div");
    elThread.id = "thread";

    const elCommentLoading = document.createElement("p");
    elCommentLoading.id = "comment-loading";
    elCommentLoading.innerText = "Loading…";
    elThread.appendChild(elCommentLoading);

    lineElement.after(elThread);

    // Get parent section.
    const section = lineElement.closest(".section") as HTMLElement;
    const sectionId = section.getAttribute("edat_section")!;
    const lineNumber = lineElement.getAttribute("edat_line")!;

    fetch(`/thread/${sectionId}/${lineNumber}`).then((res) => res.text().then(html => {
        elThread.innerHTML = html;
        universal.processUtcs();
        elThread.scrollIntoView({
            behavior: "smooth",
            block: "center",
        });

        const elCommentInstructions = elThread
            .querySelector("#comment-instructions") as HTMLElement;

        (elThread.querySelector("#close-comments") as HTMLElement).onclick = hideComments;

        for (const el of elThread.querySelectorAll(".remove") as NodeListOf<HTMLElement>) {
            el.onclick = () => {
                const uuid = el.getAttribute("edat_uuid")!;
                
                hideComments();
    
                fetch(`/remove_comment/${sectionId}/${uuid}`, {
                    method: "DELETE",
                }).then(() => {
                    showComments(lineElement);
                });
            }
        }

        for (const el of elThread.querySelectorAll(".unremove") as NodeListOf<HTMLElement>) {
            el.onclick = () => {
                const uuid = el.getAttribute("edat_uuid")!;
                
                hideComments();
    
                fetch(`/unremove_comment/${sectionId}/${uuid}`, {
                    method: "POST",
                }).then(() => {
                    showComments(lineElement);
                });
            }
        }

        for (const el of elThread.querySelectorAll(".edit") as NodeListOf<HTMLElement>) {
            el.onclick = () => {
                const myUuid = el.getAttribute("edat_uuid")!;

                if (editUuid == myUuid) {
                    el.innerText = "Edit";
                    editUuid = null;
                    const elEditing = document.querySelector(".editing") as HTMLElement;
                    elEditing.classList.remove("editing");
                    elUserComment.innerText = "";
                    return;
                }

                
                el.innerText = "Cancel edit";
                elCommentInstructions.innerText = "Editing highlighted comment…";
                
                const elEditing = document.querySelector(".editing") as HTMLElement | null;
                if (elEditing) {
                    elEditing.classList.remove("editing");
                }
                el.closest(".comment")!.classList.add("editing");
                
                elUserComment.innerText
                    = (el.closest(".comment")!.querySelector(".text")! as HTMLElement).innerText;
                
                editUuid = myUuid;
            }
        }

        const elUserComment = document.getElementById("user-comment") as HTMLTextAreaElement;
        if (elUserComment) {
            elUserComment.onkeydown = ev => {
                if (ev.key == "Enter") {
                    ev.preventDefault();
    
                    if (elUserComment.value.length == 0) {
                        return;
                    }
                    
                    hideComments();

                    if (editUuid) {
                        fetch(`/edit_comment/${sectionId}/${editUuid}`, {
                            method: "POST",
                            body: elUserComment.value,
                        }).then(() => {
                            showComments(lineElement);
                        });
                    } else {
                        fetch(`/comment/${sectionId}/${lineNumber}`, {
                            method: "POST",
                            body: elUserComment.value,
                        }).then(() => {
                            showComments(lineElement);
                        });
                    }
                }
            };
        }
    }));
}

function hideComments() {
    const elThread = document.getElementById("thread") as HTMLElement;
    elThread.style.display = "none";

    openComment = null;
    elThread.remove();
}

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
let sectionProgresses: Progress[] = [];
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
            sectionProgresses.push({
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
        for (const section of sectionProgresses) {
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
    for (const section of sectionProgresses) {
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
