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
