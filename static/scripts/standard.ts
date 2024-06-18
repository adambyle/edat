const elTitle = document.getElementById("title") as HTMLHeadingElement;
const elTitleSpan = elTitle.children[0] as HTMLSpanElement;

const elPageTitle = document.getElementsByClassName("page-title")[0] as HTMLElement;
let pageTitleShown = false;
let pageTitleTimeout: number;
if (elPageTitle) {
    addEventListener("scroll", () => {
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
    scrollTo({
        left: 0,
        top: 0,
        behavior: "smooth",
    });
}

export function titleClick(handler: () => void) {
    elTitle.onclick = handler;
}

let scrolledPast = false;
addEventListener("scroll", () => {
    if (elTitle.getBoundingClientRect().top <= -10 && !scrolledPast) {
        document.body.classList.add("scrolled-past");
        scrolledPast = true;
    } else if (scrollY < 10 && scrolledPast) {
        scrolledPast = false;
        document.body.classList.remove("scrolled-past");
    }
});

const elDrawer = document.getElementById("drawer") as HTMLDivElement;
const elDrawerNotification = document
    .querySelector("#drawer .notification") as HTMLParagraphElement;
let notificationShowing = false;
let contentShowing = false;
let hideNotification: number;
let drawerCloseEvent = () => {};

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
            document.body.style.overflow = "scroll";
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
        document.body.style.overflow = "hidden";
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
    document.body.style.overflow = "hidden";
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
    document.body.style.overflow = "scroll";
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

for (const el of document.getElementsByClassName("drawer-close")) {
    if (el instanceof HTMLElement) {
        el.onclick = closeDrawer;
    }
}
