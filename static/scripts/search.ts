import "./universal.js";

const elBody = document.querySelector(".body") as HTMLDivElement;
const searchInput = document.getElementById("search-input") as HTMLInputElement;
const elResultName = document.getElementById("result-name") as HTMLSpanElement;
const elSearchWrapper = document.getElementById("search-wrapper") as HTMLDivElement;
const elTitle = document.querySelector("h1")!;
const elResultsCarousel = document.querySelector(".results.carousel") as HTMLDivElement;

addEventListener("scroll", () => {
    if (elResultsCarousel.getBoundingClientRect().top
        < elSearchWrapper.getBoundingClientRect().bottom) {
        elSearchWrapper.style.borderBottomWidth = "1px";
    } else {
        elSearchWrapper.style.borderBottomWidth = "0";;
    }
});

const results = document.getElementsByClassName("result") as HTMLCollectionOf<HTMLDivElement>;
const firstResult = results[0];
if (firstResult) {
    firstResult.classList.add("selected");
    const firstSubresult = document
        .querySelector(".subresults.carousel .subresult") as HTMLDivElement;
    firstSubresult.classList.add("selected");

    showBody(firstSubresult);
}
const subresults = document.getElementsByClassName("subresult") as HTMLCollectionOf<HTMLDivElement>;
const elSubresultsCarousel = document.querySelector(".subresults.carousel") as HTMLDivElement;

for (const result of results) {
    result.onclick = () => {
        document.querySelector(".result.selected")!.classList.remove("selected");
        result.classList.add("selected");
        setTimeout(() => {
            result.scrollIntoView({ behavior: "smooth", block: "center" });
        }, 100);
        const mySubresults = result.querySelector(".mysubresults") as HTMLDivElement;
        elSubresultsCarousel.innerHTML = mySubresults.innerHTML;
        elResultName.innerHTML = result.querySelector("h4")!.innerHTML;

        const firstSubresult = elSubresultsCarousel.querySelector(".subresult") as HTMLDivElement;
        firstSubresult.classList.add("selected");
        setTimeout(() => {
            firstSubresult.scrollIntoView({ behavior: "smooth", block: "center" });
        }, 500);
        showBody(firstSubresult);

        for (const subresult
            of elSubresultsCarousel.children as HTMLCollectionOf<HTMLDivElement>) {
            subresult.onclick = () => {
                document.querySelector(".subresult.selected")!.classList.remove("selected");
                subresult.classList.add("selected");
                setTimeout(() => {
                    subresult.scrollIntoView({ behavior: "smooth", block: "center" });
                }, 100);
                showBody(subresult);
            }
        }
    };
}

for (const subresult of subresults) {
    subresult.onclick = () => {
        document.querySelector(".subresult.selected")!.classList.remove("selected");
        subresult.classList.add("selected");
        setTimeout(() => {
            subresult.scrollIntoView({ behavior: "smooth", block: "center" });
        }, 100);
        showBody(subresult);
    }
}

let reshowTitle: number;
function showBody(subresult: HTMLDivElement) {
    const searchPath = subresult.getAttribute("edat_search")!;
    elBody.innerHTML = `
        <p class="loading">Loading results</p>
    `;
    const words = searchInput.value.split(" ").filter(s => s.length > 0);
    const search = words.join(",");
    fetch(`/search/${searchPath}/${search}`).then(res => res.text().then(html => {
        elBody.innerHTML = html;

        const allMatches = document
            .getElementsByClassName("allmatch") as HTMLCollectionOf<HTMLElement>;
        const jumps = document.getElementsByClassName("jump") as HTMLCollectionOf<HTMLElement>;

        for (let i = 0; i < allMatches.length; i++) {
            jumps[i].onclick = () => {
                allMatches[i].scrollIntoView({ behavior: "smooth", block: "center" });
            }
        }
    }));
}

searchInput.onkeydown = (ev) => {
    if (ev.key == "Enter" && searchInput.value.length > 0) {
        const words = searchInput.value.split(" ").filter(s => s.length > 0);
        const search = words.join(",");
        location.href = `/search/${search}`;
    }
}
