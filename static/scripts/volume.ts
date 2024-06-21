import "./universal.js";
import "./standard.js";
import * as standard from "./standard.js";

const elUnreadDrawer = document.getElementById("unread-drawer") as HTMLDivElement | null;
if (elUnreadDrawer) {
    setTimeout(() => {
        function promptUnread() {
            standard.drawerNotification("See your unread and unfinished entries", null, elUnreadDrawer!, promptUnread);
        }

        promptUnread();
    }, 500);
}

const markAsReadButtons = document
    .getElementsByClassName("skip") as HTMLCollectionOf<HTMLButtonElement>;

for (const button of markAsReadButtons) {
    button.onclick = () => {
        button.innerText = "Marked as read";
        let id = Number.parseInt(button.getAttribute("edat_section")!);
        button.style.color = "var(--content)";

        fetch(`/read/${id}`, { method: "POST" });
    };
}
