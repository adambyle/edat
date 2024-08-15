import * as universal from "./universal.js";

type Position<C, I> =
    | { "StartOf": C }
    | { "Before": I }
    | { "After": I }
    | { "EndOf": C };

type ContentStatus = "Missing" | "Incomplete" | "Complete";
type UserPrivilege = "Owner" | "Reader";

type ContentType =
    | "Journal"
    | "Archive"
    | "Diary"
    | "Cartoons"
    | "Creative"
    | "Featured";

const contentStatuses: ContentStatus[] = ["Missing", "Incomplete", "Complete"];
const userPrivileges: UserPrivilege[] = ["Owner", "Reader"];
const contentTypes: ContentType[] = [
    "Journal",
    "Archive",
    "Diary",
    "Cartoons",
    "Creative",
    "Featured",
];

function capitalize(str: string): string {
    return str[0].toUpperCase() + str.substring(1);
}

type Cmd =
    | {
        GetSection: {
            id: number,
        },
    }
    | {
        NewSection: {
            date: string
        }
    }
    | {
        SetSection: {
            id: number,
            heading: string,
            description: string,
            summary: string,
            date: string,
        }
    }
    | {
        SetNewSection: {
            position: Position<string, number>,
            heading: string,
            description: string,
            summary: string,
            date: string,
        }
    }
    | {
        DeleteSection: {
            id: number,
        }
    }
    | {
        MoveSection: {
            id: number,
            position: Position<string, number>,
        }
    }
    | {
        SectionStatus: {
            id: number,
            status: ContentStatus,
        }
    }
    | {
        GetEntry: {
            id: string,
        }
    }
    | "NewEntry"
    | {
        SetEntry: {
            id: string,
            title: string,
            description: string,
            summary: string,
        }
    }
    | {
        SetNewEntry: {
            title: string,
            position: Position<[string, number], string>,
            description: string,
            summary: string,
        }
    }
    | {
        DeleteEntry: {
            id: string,
        }
    }
    | {
        MoveEntry: {
            id: string,
            position: Position<[string, number], string>,
        }
    }
    | {
        GetVolume: {
            id: string,
        },
    }
    | "NewVolume"
    | {
        SetVolume: {
            id: string,
            title: string,
            subtitle: string,
        }
    }
    | {
        SetNewVolume: {
            position: Position<null, string>,
            title: string,
            subtitle: string,
        }
    }
    | {
        DeleteVolume: {
            id: string,
        }
    }
    | {
        MoveVolume: {
            id: string,
            position: Position<null, string>,
        }
    }
    | {
        VolumeContentType: {
            id: string,
            content_type: ContentType,
        }
    }
    | {
        GetUser: {
            id: string,
        }
    }
    | {
        SetUser: {
            id: string,
            first_name: string,
            last_name: string,
        },
    }
    | {
        SetNewUser: {
            first_name: string,
            last_name: string,
        }
    }
    | "NewUser"
    | {
        UserPrivilege: {
            id: string,
            privilege: UserPrivilege,
        }
    }
    | {
        AddUserCode: {
            id: string,
            code: string,
        }
    }
    | {
        RemoveUserCode: {
            id: string,
            code: string,
        },
    }
    | "Volumes"
    | "NextSectionId"
    | "Images"
    | {
        GetIntro: {
            id: string | null,
        }
    }
    | {
        SetIntro: {
            id: string | null,
            content: string,
        }
    }
    | {
        GetContent: {
            id: number,
        }
    }
    | {
        SetContent: {
            id: number,
            content: string,
        }
    }
    | {
        InitUser: {
            id: string,
        }
    }
    | "NewReview"
    | {
        SetNewReview: SetNewReviewBody,
    }
    | {
        SetTrackReview: {
            track_id: string,
            score: number,
        }
    }
    | "NewMonthInReview"
    | {
        SetMonthInReview: {
            albums: string[],
            tracks: string[],
            month: number,
            year: number,
        }
    };

interface SetNewReviewBody {
    album_id: string,
    genre?: string,
    score?: number,
    review?: string,
    summary?: string,
    first_listened?: string,
}

const commandInput = document.getElementById("command") as HTMLInputElement;
const elInvalidCommand = document.getElementById("invalid-command") as HTMLParagraphElement;
const elResponse = document.getElementById("response") as HTMLDivElement;

commandInput.onkeydown = (ev) => {
    elInvalidCommand.style.opacity = "0.0";

    if (ev.key == "Enter") {
        parseCommand(commandInput.value);
    }
}

function parseCommand(command: string) {
    const args = command.split(" ").map(arg => arg.trim()).filter(arg => arg.length > 0);

    function expectArgs(count: number): boolean {
        if (args.length < count) {
            parseError();
            return false;
        } else {
            return true;
        }
    }

    const root = args[0].toLowerCase();
    if (root == "privilege") {
        if (!expectArgs(3)) {
            return;
        }
        const user = args[1];
        const privilege = capitalize(args[2]) as UserPrivilege;
        if (!userPrivileges.includes(privilege)) {
            parseError();
            return;
        }

        submitAction = updateUser(user);
        cmd({
            UserPrivilege: {
                id: user,
                privilege,
            },
        });
    } else if (root == "status") {
        if (!expectArgs(3)) {
            return;
        }
        const section = Number.parseInt(args[1]);
        if (!isNumber(section)) {
            parseError();
            return;
        }
        const status = capitalize(args[2]) as ContentStatus;
        if (!contentStatuses.includes(status)) {
            parseError();
            return;
        }

        submitAction = updateSection(section);
        cmd({
            SectionStatus: {
                id: section,
                status,
            },
        });
    } else if (root == "volumetype") {
        if (!expectArgs(3)) {
            return;
        }
        const volume = args[1];
        const contentType = capitalize(args[2]) as ContentType;
        if (!contentTypes.includes(contentType)) {
            parseError();
            return;
        }

        submitAction = updateVolume(volume);
        cmd({
            VolumeContentType: {
                id: volume,
                content_type: contentType,
            },
        });
    } else if (root == "new") {
        if (!expectArgs(2)) {
            return;
        }
        const positionArgs = args.slice(2);
        switch (args[1]) {
            case "section":
                {
                    const position = parseSectionPosition(positionArgs);
                    if (position[0] == null) {
                        parseError();
                        return;
                    }
                    submitAction = newSection(position[0]);
                    cmd({
                        NewSection: {
                            date: universal.nowString(),
                        },
                    });
                    break;
                }
            case "entry":
                {
                    const position = parseEntryPosition(positionArgs);
                    if (position[0] == null) {
                        parseError();
                        return;
                    }
                    submitAction = newEntry(position[0]);
                    cmd("NewEntry");
                    break;
                }
            case "volume":
                {
                    const position = parseVolumePosition(positionArgs);
                    if (position[0] == null) {
                        parseError();
                        return;
                    }
                    submitAction = newVolume(position[0]);
                    cmd("NewVolume");
                    break;
                }
            case "user":
                submitAction = newUser;
                cmd("NewUser");
                break;
            default:
                parseError();
                return;
        }
    } else if (root == "move") {
        if (!expectArgs(3)) {
            return;
        }
        const id = args[2];
        const positionArgs = args.slice(3);
        switch (args[1]) {
            case "section":
                {
                    const position = parseSectionPosition(positionArgs);
                    if (position[0] == null) {
                        parseError();
                        return;
                    }
                    const section = Number.parseInt(id);
                    if (!isNumber(section)) {
                        parseError();
                        return;
                    }
                    submitAction = updateSection(section);
                    cmd({
                        MoveSection: {
                            id: section,
                            position: position[0],
                        }
                    });
                    break;
                }
            case "entry":
                {
                    const position = parseEntryPosition(positionArgs);
                    if (position[0] == null) {
                        parseError();
                        return;
                    }
                    submitAction = updateEntry(id);
                    cmd({
                        MoveEntry: {
                            id,
                            position: position[0],
                        }
                    });
                    break;
                }
            case "volume":
                {
                    const position = parseVolumePosition(positionArgs);
                    if (position[0] == null) {
                        parseError();
                        return;
                    }
                    submitAction = updateVolume(id);
                    cmd({
                        MoveVolume: {
                            id,
                            position: position[0],
                        }
                    });
                    break;
                }
            default:
                parseError();
                return;
        }
    } else if (root == "get") {
        if (!expectArgs(3)) {
            return;
        }
        const id = args[2];
        switch (args[1]) {
            case "section":
                {
                    const section = Number.parseInt(id);
                    if (!isNumber(section)) {
                        parseError();
                        return;
                    }
                    submitAction = updateSection(section);
                    cmd({
                        GetSection: {
                            id: section,
                        }
                    });
                    break;
                }
            case "entry":
                submitAction = updateEntry(id);
                cmd({
                    GetEntry: {
                        id,
                    }
                });
                break;
            case "volume":
                submitAction = updateVolume(id);
                cmd({
                    GetVolume: {
                        id,
                    }
                });
                break;
            case "user":
                submitAction = updateUser(id);
                cmd({
                    GetUser: {
                        id,
                    }
                });
                break;
            default:
                parseError();
                return;
        }
    } else if (root == "delete") {
        if (!expectArgs(3)) {
            return;
        }
        const id = args[2];
        switch (args[1]) {
            case "section":
                {
                    const section = Number.parseInt(id);
                    if (!isNumber(section)) {
                        parseError();
                        return;
                    }
                    submitAction = updateSection(section);
                    cmd({
                        DeleteSection: {
                            id: section,
                        }
                    });
                    break;
                }
            case "entry":
                submitAction = updateEntry(id);
                cmd({
                    DeleteEntry: {
                        id,
                    }
                });
                break;
            case "volume":
                submitAction = updateVolume(id);
                cmd({
                    DeleteVolume: {
                        id,
                    }
                });
                break;
            default:
                parseError();
        }
    } else if (root == "volumes") {
        cmd("Volumes");
    } else if (root == "code") {
        if (!expectArgs(4)) {
            return;
        }
        const user = args[2];
        const code = args[3];
        if (args[1] == "add") {
            submitAction = updateUser(user);
            cmd({
                AddUserCode: {
                    id: user,
                    code,
                },
            });
        } else if (args[1] == "remove") {
            submitAction = updateUser(user);
            cmd({
                RemoveUserCode: {
                    id: user,
                    code,
                },
            });
        } else {
            parseError();
        }
    } else if (root == "images") {
        cmd("Images");
    } else if (root == "intro") {
        if (!expectArgs(2)) {
            parseError();
            return;
        }
        let volume: string | null = args[1];
        if (volume == "edat") {
            volume = null;
        }
        submitAction = updateIntro(volume);
        cmd({
            GetIntro: {
                id: volume,
            },
        });
    } else if (root == "content") {
        if (!expectArgs(2)) {
            parseError();
            return;
        }
        const section = Number.parseInt(args[1]);
        if (!isNumber(section)) {
            parseError();
            return;
        }
        submitAction = updateContent(section);
        cmd({
            GetContent: {
                id: section,
            },
        });
    } else if (root == "init") {
        if (!expectArgs(2)) {
            return;
        }
        const user = args[1];
        submitAction = updateUser(user);
        cmd({
            InitUser: {
                id: user,
            },
        });
    } else if (root == "review") {
        if (!expectArgs(2)) {
            return;
        }
        const mode = args[1];
        if (mode == "album") {
            submitAction = newReview;
            cmd("NewReview");
        } else if (mode == "track") {
            if (!expectArgs(4)) {
                return;
            }
            const trackId = args[2];
            const score = Number.parseInt(args[3]);
            if (!Number.isNaN(score)) {
                cmd({
                    SetTrackReview: {
                        track_id: trackId,
                        score,
                    }
                });
            }
        } else if (mode == "month") {
            submitAction = newMonthInReview;
            cmd("NewMonthInReview");
        }
    } else {
        parseError();
    }
}

function updateIntro(id: string | null) {
    return () => {
        const elContents = document.getElementById("contents") as HTMLInputElement;

        if (elContents.value.length > 0) {
            submitAction = updateIntro(id);
            cmd({
                SetIntro: {
                    id,
                    content: elContents.value,
                }
            });
        }
    };
}

function updateContent(id: number) {
    return () => {
        const elContents = document.getElementById("contents") as HTMLInputElement;

        if (elContents.value.length > 0) {
            submitAction = updateContent(id);
            cmd({
                SetContent: {
                    id,
                    content: elContents.value,
                }
            });
        }
    };
}

function newUser() {
    const firstNameInput = document.getElementById("user-first-name") as HTMLInputElement;
    const lastNameInput = document.getElementById("user-last-name") as HTMLInputElement;

    if (firstNameInput.value.length > 0 && lastNameInput.value.length > 0) {
        const newId = createId(firstNameInput.value + lastNameInput.value);
        submitAction = updateUser(newId);
        cmd({
            SetNewUser: {
                first_name: firstNameInput.value,
                last_name: lastNameInput.value,
            },
        });
    }
}

function newVolume(position: Position<null, string>) {
    return () => {
        const titleInput = document.getElementById("volume-title") as HTMLInputElement;
        const subtitleInput = document.getElementById("volume-subtitle") as HTMLInputElement;

        if (titleInput.value.length > 0) {
            const newId = createId(titleInput.value);
            submitAction = updateVolume(newId);
            cmd({
                SetNewVolume: {
                    title: titleInput.value,
                    subtitle: subtitleInput.value,
                    position,
                },
            });
        }
    };
}

function newEntry(position: Position<[string, number], string>) {
    return () => {
        const titleInput = document.getElementById("entry-title") as HTMLInputElement;
        const descriptionInput = document.getElementById("entry-description") as HTMLInputElement;
        const summaryInput = document.getElementById("entry-summary") as HTMLInputElement;

        if (titleInput.value.length > 0
            && descriptionInput.value.length > 0
            && summaryInput.value.length > 0) {
            const newId = createId(titleInput.value);
            submitAction = updateEntry(newId);
            cmd({
                SetNewEntry: {
                    title: titleInput.value,
                    description: descriptionInput.value,
                    summary: summaryInput.value,
                    position,
                },
            });
        }
    };
}

function newReview() {
    const albumId = document.getElementById("album-id") as HTMLInputElement;
    const albumGenre = document.getElementById("album-genre") as HTMLInputElement;
    const albumScore = document.getElementById("album-score") as HTMLInputElement;
    const albumReview = document.getElementById("contents") as HTMLInputElement;
    const albumSummary = document.getElementById("album-summary") as HTMLInputElement;
    const albumListenDate = document.getElementById("album-listen-date") as HTMLInputElement;

    if (albumId.value.length > 0) {
        submitAction = () => {};

        let reviewDetails: SetNewReviewBody = {
            album_id: albumId.value,
        };

        const genre = albumGenre.value.trim();
        if (genre.length > 0) {
            reviewDetails.genre = genre;
        }

        const score = Number.parseInt(albumScore.value);
        if (!Number.isNaN(score)) {
            reviewDetails.score = score;
        }

        const review = albumReview.value.trim();
        if (review.length > 0) {
            reviewDetails.review = review;
        }

        const summary = albumSummary.value.trim();
        if (summary.length > 0) {
            reviewDetails.summary = summary;
        }

        const listenDate = albumListenDate.value.trim();
        if (listenDate.length > 0) {
            reviewDetails.first_listened = listenDate;
        }

        cmd({
            SetNewReview: reviewDetails,
        });
    }
}

function newMonthInReview() {
    const reviewAlbums = document.getElementById("review-albums") as HTMLTextAreaElement;
    const reviewTracks = document.getElementById("review-tracks") as HTMLTextAreaElement;
    const reviewMonth = document.getElementById("review-month") as HTMLInputElement;

    const albums = reviewAlbums.value.trim().split(" ").filter(a => a.length > 0);
    const tracks = reviewTracks.value.trim().split(" ").filter(t => t.length > 0);
    const dateSplit = reviewMonth.value.split("-");
    if (dateSplit.length != 2) {
        return;
    }
    const year = Number.parseInt(dateSplit[0]);
    const month = Number.parseInt(dateSplit[1]);
    if (Number.isNaN(year) || Number.isNaN(month)) {
        return;
    }
    if (albums.length == 0 || tracks.length == 0) {
        return;
    }

    submitAction = () => {};

    cmd({
        SetMonthInReview: {
            albums,
            tracks,
            year,
            month,
        }
    });
}

function newSection(position: Position<string, number>) {
    return () => {
        const headingInput = document.getElementById("section-heading") as HTMLInputElement;
        const descriptionInput = document.getElementById("section-description") as HTMLInputElement;
        const summaryInput = document.getElementById("section-summary") as HTMLInputElement;
        const dateInput = document.getElementById("section-date") as HTMLInputElement;

        if (descriptionInput.value.length > 0 && summaryInput.value.length > 0) {
            fetch(
                "/cmd",
                {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify("NextSectionId"),
                }
            ).then(res => res.json().then(nextId => {
                submitAction = updateSection(nextId);
                cmd({
                    SetNewSection: {
                        heading: headingInput.value,
                        description: descriptionInput.value,
                        summary: summaryInput.value,
                        date: dateInput.value,
                        position,
                    },
                });
            }));
        }
    };
}

function updateUser(id: string) {
    return () => {
        const firstNameInput = document.getElementById("user-first-name") as HTMLInputElement;
        const lastNameInput = document.getElementById("user-last-name") as HTMLInputElement;

        if (firstNameInput.value.length > 0 && lastNameInput.value.length > 0) {
            const newId = createId(firstNameInput.value + lastNameInput.value);
            submitAction = updateUser(newId);
            cmd({
                SetUser: {
                    first_name: firstNameInput.value,
                    last_name: lastNameInput.value,
                    id,
                },
            });
        }
    };
}

function updateVolume(id: string) {
    return () => {
        const titleInput = document.getElementById("volume-title") as HTMLInputElement;
        const subtitleInput = document.getElementById("volume-subtitle") as HTMLInputElement;

        if (titleInput.value.length > 0) {
            const newId = createId(titleInput.value);
            submitAction = updateVolume(newId);
            cmd({
                SetVolume: {
                    title: titleInput.value,
                    subtitle: subtitleInput.value,
                    id,
                },
            });
        }
    };
}

function updateEntry(id: string) {
    return () => {
        const titleInput = document.getElementById("entry-title") as HTMLInputElement;
        const descriptionInput = document.getElementById("entry-description") as HTMLInputElement;
        const summaryInput = document.getElementById("entry-summary") as HTMLInputElement;

        if (titleInput.value.length > 0
            && descriptionInput.value.length > 0
            && summaryInput.value.length > 0) {
            const newId = createId(titleInput.value);
            submitAction = updateEntry(newId);
            cmd({
                SetEntry: {
                    title: titleInput.value,
                    description: descriptionInput.value,
                    summary: summaryInput.value,
                    id,
                },
            });
        }
    };
}

function updateSection(id: number) {
    return () => {
        const headingInput = document.getElementById("section-heading") as HTMLInputElement;
        const descriptionInput = document.getElementById("section-description") as HTMLInputElement;
        const summaryInput = document.getElementById("section-summary") as HTMLInputElement;
        const dateInput = document.getElementById("section-date") as HTMLInputElement;

        if (descriptionInput.value.length > 0 && summaryInput.value.length > 0) {
            submitAction = updateSection(id);
            cmd({
                SetSection: {
                    heading: headingInput.value,
                    description: descriptionInput.value,
                    summary: summaryInput.value,
                    id,
                    date: dateInput.value,
                },
            });
        }
    };
}

function parseVolumePosition(positionArgs: string[]): [Position<null, string> | null, number] {
    if (positionArgs.length < 1) {
        return [null, 0];
    }
    switch (positionArgs[0]) {
        case "start":
            return [
                { "StartOf": null },
                1,
            ];
        case "end":
            return [
                { "EndOf": null },
                1,
            ];
        case "after":
            if (positionArgs.length < 2) {
                return [null, 0];
            }
            return [
                { "After": positionArgs[1] },
                2,
            ];
        case "before":
            if (positionArgs.length < 2) {
                return [null, 0];
            }
            return [
                { "Before": positionArgs[1] },
                2,
            ];
        default:
            return [null, 0];
    }
}

function parseEntryPosition(
    positionArgs: string[]
): [Position<[string, number], string> | null, number] {
    if (positionArgs.length < 2) {
        return [null, 0];
    }
    switch (positionArgs[0]) {
        case "startof":
            {
                if (positionArgs.length < 3) {
                    return [null, 0];
                }
                const volumePart = Number.parseInt(positionArgs[2]);
                if (!isNumber(volumePart)) {
                    return [null, 0];
                }
                return [
                    { "StartOf": [positionArgs[1], volumePart] },
                    3,
                ];
            }
        case "endof":
            {
                if (positionArgs.length < 3) {
                    return [null, 0];
                }
                const volumePart = Number.parseInt(positionArgs[2]);
                if (!isNumber(volumePart)) {
                    return [null, 0];
                }
                return [
                    { "EndOf": [positionArgs[1], volumePart] },
                    3,
                ];
            }
        case "after":
            return [
                { "After": positionArgs[1] },
                2,
            ];
        case "before":
            return [
                { "Before": positionArgs[1] },
                2,
            ];
        default:
            return [null, 0];
    }
}

function parseSectionPosition(positionArgs: string[]): [Position<string, number> | null, number] {
    if (positionArgs.length < 2) {
        return [null, 0];
    }
    switch (positionArgs[0]) {
        case "startof":
            return [
                { "StartOf": positionArgs[1] },
                2,
            ];
        case "endof":
            return [
                { "EndOf": positionArgs[1] },
                2,
            ];
        case "after":
            {
                const section = Number.parseInt(positionArgs[1]);
                if (!isNumber(section)) {
                    return [null, 0];
                }
                return [
                    { "After": section },
                    2,
                ];
            }
        case "before":
            {
                const section = Number.parseInt(positionArgs[1]);
                if (!isNumber(section)) {
                    return [null, 0];
                }
                return [
                    { "Before": section },
                    2,
                ];
            }
        default:
            return [null, 0];
    }
}

function isNumber(n: number): boolean {
    return !Number.isNaN(n) && n >= 0 && n != Infinity;
}

function createId(name: string) {
    return name
        .trim()
        .toLowerCase()
        .replaceAll("<i>", "")
        .replaceAll("&", "and")
        .replaceAll("</i>", "")
        .replaceAll(" ", "-");
}

function parseError() {
    elInvalidCommand.style.opacity = "1.0";
}

let submitAction: () => void;

function handlePaste(
    element: HTMLTextAreaElement, 
    processing: HTMLElement,
    ev: ClipboardEvent
) {
    ev.preventDefault();
    const html = ev.clipboardData?.getData("text/html");
    const text = ev.clipboardData?.getData("text/plain");

    if (html && html.length > 0) {
        const selectionStart = element.selectionStart!;
        const selectionEnd = element.selectionEnd!;
        const before = element.value.substring(0, selectionStart);
        const after = element.value.substring(selectionEnd);

        let output = "";
        processing.innerHTML = html;
        for (const p of processing.querySelectorAll("p")) {
            let text = "";
            for (const span of p.children as HTMLCollectionOf<HTMLSpanElement>) {
                if (span.style.fontStyle == "italic") {
                    text += "<i>";
                }
                text += `${span.innerText}`;
                if (span.style.fontStyle == "italic") {
                    text += "</i>";
                }
            }
            output += text;
            output += "\n";
        }

        element.value = `${before}${output}${after}`;
        element.setSelectionRange(selectionStart + html.length, selectionStart + html.length);
    } else if (text && text.length > 0) {
        const selectionStart = element.selectionStart!;
        const selectionEnd = element.selectionEnd!;
        const before = element.value.substring(0, selectionStart);
        const after = element.value.substring(selectionEnd);
        element.value = `${before}${text}${after}`;
        element.setSelectionRange(selectionStart + text.length, selectionStart + text.length);
    }
}

function cmd(command: Cmd) {
    elResponse.innerHTML = "";
    fetch("/cmd", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(command),
    }).then(res => res.text().then(text => {
        elResponse.innerHTML = text;
        universal.processUtcs();
        const submitButton = document.getElementById("submit");
        if (submitButton instanceof HTMLButtonElement) {
            submitButton.onclick = submitAction;
        }

        const elContents = document.getElementById("contents") as HTMLTextAreaElement;
        if (elContents) {
            const elProcessing = document.getElementById("processing") as HTMLDivElement;
            elContents.onpaste = (ev) => handlePaste(elContents, elProcessing, ev);
        }

        const elImage = document.getElementById("image") as HTMLImageElement | null;
        if (elImage) {
            // Image console handling.
            const imageIdInput = document.getElementById("image-id") as HTMLInputElement;
            const imageUploadInput = document.getElementById("image-upload") as HTMLInputElement;
            const uploadButton = document.getElementById("upload") as HTMLButtonElement;
            const elImageFeedback = document.getElementById("image-feedback") as HTMLDivElement;

            imageIdInput.onkeydown = (ev) => {
                if (ev.key == "Enter") {
                    elImage.src = `/image/${imageIdInput.value}.jpg`;
                }
            }

            uploadButton.addEventListener("click", () => {
                let files = imageUploadInput.files!;
                let totalFiles = files.length;
                let done = 0;

                for (const file of files) {
                    fetch(
                        `/image/${file.name}`,
                        {
                            method: "POST",
                            headers: { "Content-Type": file.type },
                            body: file,
                        }
                    ).then(res => res.text().then(text => {
                        elImageFeedback.innerHTML += text;
                        done += 1;
                        if (done == totalFiles) {
                            imageUploadInput.value = "";
                        }
                    }));
                }
            });
        }
    }));
}
