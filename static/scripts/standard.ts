const elTitle = document.getElementById("title") as HTMLHeadingElement;
const elTitleSpan = elTitle.children[0] as HTMLSpanElement;
const elHeader = document.getElementById("header") as HTMLDivElement;

function showAltTitle() {
    pageTitleShown = true;
    clearTimeout(pageTitleTimeout);
    elTitleSpan.style.opacity = "0.0";
    pageTitleTimeout = setTimeout(() => {
        elTitleSpan.classList.add("page-title-ref");
        elTitleSpan.innerHTML = elPageTitle.innerHTML;
        elTitleSpan.classList.add("forcesmall");
        elTitleSpan.style.opacity = "1.0";
    }, 100);
}

function hideAltTitle() {
    pageTitleShown = false;
    clearTimeout(pageTitleTimeout);
    elTitleSpan.style.opacity = "0.0";
    pageTitleTimeout = setTimeout(() => {
        elTitleSpan.classList.remove("page-title-ref");
        elTitleSpan.innerHTML = "Every Day’s a Thursday";
        elTitleSpan.classList.remove("forcesmall");
        elTitleSpan.style.opacity = "1.0";
    }, 100);
}

const elPageTitle = document.getElementsByClassName("page-title")[0] as HTMLElement;
let pageTitleShown = false;
let pageTitleTimeout: number;
if (elPageTitle) {
    addEventListener("scroll", () => {
        let pageTitleBottom = elPageTitle.getBoundingClientRect().bottom;
        let headerBottom = elTitle.getBoundingClientRect().bottom;
        if ((pageTitleBottom < headerBottom || topDrawerOpen) && !pageTitleShown) {
            showAltTitle();
        }
        if (pageTitleBottom > headerBottom && !topDrawerOpen && pageTitleShown) {
            hideAltTitle();
        }
    });
}

elTitle.onclick = () => {
    scrollTo({
        left: 0,
        top: 0,
        behavior: "smooth",
    });
}

let scrolledPast = false;
function handleHeader() {
    if (elTitle.getBoundingClientRect().top <= -10 && !scrolledPast) {
        document.body.classList.add("scrolled-past");
        scrolledPast = true;
    } else if (scrollY < 10 && scrolledPast) {
        scrolledPast = false;
        document.body.classList.remove("scrolled-past");
    }
}
addEventListener("scroll", handleHeader);
handleHeader();

// Top drawer handling.
const elTopdrawer = document.getElementById("topdrawer") as HTMLDivElement | null;
let topDrawerOpen = false;

export let closeHeader = () => {};

let onHeaderOpen = () => {};
export function setOnHeaderOpen(fn: () => void) {
    onHeaderOpen = fn;
}

if (elTopdrawer) {
    function openTopDrawer() {
        elHeader.classList.add("open");
        topDrawerOpen = true;
        lockScroll();
        if (elPageTitle && !pageTitleShown) {
            showAltTitle();
        }
        elTopdrawer!.style.display = "flex";
        setTimeout(() => {
            elTopdrawer!.style.opacity = "1.0";
            onHeaderOpen();
        }, 20);
    }

    function closeTopDrawer() {
        topDrawerOpen = false;
        elHeader.classList.remove("open");
        let headerBottom = elTitle.getBoundingClientRect().bottom;
        let pageTitleBottom = elPageTitle.getBoundingClientRect().bottom;
        if (elPageTitle && pageTitleBottom > headerBottom) {
            hideAltTitle();
        }
        elTopdrawer!.style.opacity = "0.0";
        setTimeout(() => {
            elTopdrawer!.style.display = "none";
            unlockScroll();
        }, 100);
    }
    closeHeader = closeTopDrawer;

    elTitle.onclick = () => {
        if (topDrawerOpen) {
            closeTopDrawer();
        } else {
            openTopDrawer();
        }
    }

    const closeTopDrawerButton = document
        .querySelector("#topdrawer .drawer-close") as HTMLButtonElement;
    closeTopDrawerButton.onclick = closeTopDrawer;

    document.body.addEventListener("click", ev => {
        if (!topDrawerOpen) {
            return;
        }
        if (ev.y > elTopdrawer.getBoundingClientRect().bottom) {
            closeTopDrawer();
        }
    });
}


// Drawers.
const elDrawer = document.getElementById("drawer") as HTMLDivElement;
const elDrawerNotification = document
    .querySelector("#drawer .notification") as HTMLParagraphElement;
let notificationShowing = false;
let contentShowing = false;
let hideNotification: number;
let drawerCloseEvent = () => { };

function clickOutEvent(ev: MouseEvent) {
    if (ev.y < elDrawer.getBoundingClientRect().top) {
        closeDrawer();
    }
}

export function drawerNotification(
    text: string,
    timeout: number | null,
    clickAction: HTMLElement | (() => void),
    closeEvent?: () => void
) {
    drawerCloseEvent = closeEvent || drawerCloseEvent;
    elDrawer.style.display = "block";
    notificationShowing = true;
    clearTimeout(hideNotification);
    elDrawerNotification.children[0].innerHTML = text;
    elDrawerNotification.style.display = "flex";
    setTimeout(() => {
        elDrawerNotification.style.transition = "opacity 0.4s ease";
        elDrawerNotification.style.opacity = "1";
    }, 10);
    setTimeout(() => {
        elDrawerNotification.style.transition = "opacity 0.1s ease";
    }, 400);
    if (timeout) {
        hideNotification = setTimeout(() => {
            elDrawerNotification.style.opacity = "0";
            notificationShowing = false;
            unlockScroll();
            setTimeout(() => {
                if (!notificationShowing) {
                    elDrawerNotification.style.display = "none";
                    if (!contentShowing) {
                        elDrawer.style.display = "none";
                    }
                }
            }, 100);
        }, timeout);
    }

    elDrawer.onclick = () => {
        elDrawer.onclick = null;
        elDrawerNotification.style.opacity = "0";
        notificationShowing = false;
        lockScroll();
        clearTimeout(hideNotification);
        if (clickAction instanceof HTMLElement) {
            contentShowing = true;
            elDrawerNotification.style.opacity = "0";
            setTimeout(() => {
                elDrawerNotification.style.display = "none";
                document.body.addEventListener("click", clickOutEvent);
                clickAction.style.display = "block";
                setTimeout(() => {
                    clickAction.style.opacity = "1";
                }, 10);
            }, 100);
        } else {
            elDrawerNotification.style.opacity = "0";
            setTimeout(() => {
                if (!contentShowing) {
                    elDrawer.style.display = "none";
                }
                if (!notificationShowing) {
                    elDrawerNotification.style.display = "none";
                }
            }, 100);
            clickAction();
        }
    };
}

export function showDrawerElement(el: HTMLElement, then?: () => void) {
    lockScroll();
    elDrawer.onclick = null;
    elDrawerNotification.style.opacity = "0";
    notificationShowing = false;
    clearTimeout(hideNotification);
    contentShowing = true;

    setTimeout(() => {
        document.body.addEventListener("click", clickOutEvent);
        el.style.display = "block";
        setTimeout(() => {
            el.style.opacity = "1";
            if (then) {
                then();
            }
        }, 10);
    }, 100);
}

export function closeDrawer() {
    unlockScroll();
    elDrawer.onclick = null;
    clearTimeout(hideNotification);
    notificationShowing = false;
    contentShowing = false;
    document.body.removeEventListener("click", clickOutEvent);

    for (const child of elDrawer.children) {
        if (child instanceof HTMLElement) {
            child.style.opacity = "0";
            setTimeout(() => {
                child.style.display = "none";
            }, 100);
        }
    }
    setTimeout(() => {
        elDrawer.style.display = "none";
        drawerCloseEvent();
    }, 100);
}

for (const el of document.querySelectorAll("#drawer .drawer-close")) {
    if (el instanceof HTMLElement) {
        el.onclick = closeDrawer;
    }
}

function unlockScroll() {
    document.body.classList.remove("lock-scroll");
}

function lockScroll() {
    document.body.classList.add("lock-scroll");
}
