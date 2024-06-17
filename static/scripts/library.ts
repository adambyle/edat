import "./universal.js";
import "./standard.js";
import * as standard from "./standard.js";

const elSearchDrawer = document.getElementById("search-drawer") as HTMLDivElement;
setTimeout(() => {
    function promptSearch() {
        standard.drawerNotification("Search the library", null, () => {
            standard.showDrawerElement(elSearchDrawer);
            searchInput.focus();
        }, promptSearch);
    }

    promptSearch();
}, 500);

const searchInput = document.getElementById("search-input") as HTMLInputElement;
const elSearchResults = document.querySelector("#search-drawer .results") as HTMLDivElement;
let searchSubmitTimeout: number;
searchInput.oninput = () => {
    clearTimeout(searchSubmitTimeout);
    elSearchResults.innerHTML = "";
    searchSubmitTimeout = setTimeout(() => {
        let search = searchInput.value.trim().split(" ").filter(s => s.length > 0);
        if (search.length > 0) {
            fetch(
                `/components/library-search/${search.join(",")}`
            ).then((res) => res.text().then((html) => {
                elSearchResults.innerHTML = html;
            }));
        }
    }, 500);
}
