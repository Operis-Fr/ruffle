import { injectRuffleAndWait, throwIfError } from "../../utils.js";
import { expect } from "chai";
import { Player, Setup } from "ruffle-core";
import { Key } from "webdriverio";

// The player element is moved, live, into the document of a same-origin iframe (as an embedder
// would move it into a popup window), either before its movie is loaded (the instance is then
// created there) or after (the instance follows the new window when the element is adopted).
// Keys pressed in that document must reach the movie, and text copied from the movie must be
// copied from that document.

declare global {
    interface Window {
        movedPlayerLog: string[];
    }
}

async function loadMovedPlayer(moment: string, swf: string) {
    await browser.url("http://localhost:4567/test_assets/js_api.html");
    await injectRuffleAndWait(browser);
    await browser.execute(
        async (moment, swf) => {
            const container = document.getElementById("test-container")!;
            const frame = document.createElement("iframe");
            frame.id = "other-document";
            frame.style.cssText = "width: 400px; height: 400px; border: 0";
            container.appendChild(frame);
            const otherDocument = frame.contentDocument!;
            otherDocument.body.style.margin = "0";
            const input = otherDocument.createElement("input");
            input.id = "inputElement";
            otherDocument.body.appendChild(input);

            const ruffle = (window.RufflePlayer as Setup.PublicAPI).newest();
            const player = ruffle!.createPlayer() as Player.PlayerElement;
            player.id = "ruffle-player";
            player.style.cssText =
                "display: block; width: 400px; height: 400px";
            container.appendChild(player);

            window.movedPlayerLog = [];
            if (moment === "before") {
                otherDocument.body.prepend(player);
            }
            await player.ruffle().load(swf as string);
            player.ruffle().traceObserver = (message) => {
                window.movedPlayerLog.push(message);
            };
            if (moment === "after") {
                otherDocument.body.prepend(player);
            }
            player.ruffle().resume();

            // With a software renderer, the hardware acceleration modal opens on the first
            // mouseover: open it and close it now, or a click may follow its link
            const shadow = player.shadowRoot!;
            shadow
                .getElementById("container")!
                .dispatchEvent(new MouseEvent("mouseover"));
            for (const close of shadow.querySelectorAll<HTMLElement>(
                ".modal:not(.hidden) .close-modal",
            )) {
                close.click();
            }
        },
        moment,
        swf,
    );
    await throwIfError(browser);
}

async function getMovedPlayerTraces(count: number): Promise<string[]> {
    await browser.waitUntil(
        async () =>
            (await browser.execute(() => window.movedPlayerLog.length)) >=
            count,
        { timeoutMsg: `Expected Ruffle to trace ${count} messages` },
    );
    return await browser.execute(
        (count) => window.movedPlayerLog.splice(0, count as number),
        count,
    );
}

/** Clicks in the moved player, at the given position from its top left corner. */
async function clickMovedPlayer(x: number, y: number) {
    const frame = await browser.$("#other-document");
    // The frame is the size of the player, whose top left corner is at the frame's
    await frame.click({ x: x - 200, y: y - 200 });
}

for (const moment of ["before", "after"]) {
    describe(`Key events reach a player moved to another document ${moment} loading`, () => {
        before("Loads the test", async () => {
            await loadMovedPlayer(
                moment,
                "/test/integration_tests/keyboard_input/test.swf",
            );
            expect(await getMovedPlayerTraces(1)).to.deep.equal([
                "Hello from Flash!",
            ]);
        });

        it("'a' key is recognised", async () => {
            await clickMovedPlayer(200, 200);

            await browser.keys("a");
            expect(await getMovedPlayerTraces(8)).to.deep.equal([
                "onKeyDown",
                "event.charCode = 97",
                "event.keyCode = 65",
                "",
                "onKeyUp",
                "event.charCode = 97",
                "event.keyCode = 65",
                "",
            ]);
        });
    });

    describe(`Text is copied from a player moved to another document ${moment} loading`, () => {
        before("Loads the test", async () => {
            await loadMovedPlayer(
                moment,
                "/test/integration_tests/context_menu/test.swf",
            );
            expect(await getMovedPlayerTraces(1)).to.deep.equal(["Loaded!"]);
        });

        it("copying text with the keyboard works", async () => {
            await clickMovedPlayer(10, 10);
            expect(await getMovedPlayerTraces(2)).to.deep.equal([
                "onMouseDown()",
                "onMouseUp()",
            ]);

            // Populate text input in Flash
            await browser.keys("t");
            expect(await getMovedPlayerTraces(2)).to.deep.equal([
                "populating text",
                "text changed: texample",
            ]);

            await clickMovedPlayer(10, 110);
            expect(await getMovedPlayerTraces(2)).to.deep.equal([
                "onMouseDown()",
                "onMouseUp()",
            ]);
            await browser.keys([Key.Ctrl, "a"]);
            await browser.keys([Key.Ctrl, "c"]);

            // Paste into an HTML input of the same document
            await browser.execute(() => {
                const frame = document.getElementById(
                    "other-document",
                ) as HTMLIFrameElement;
                frame.contentDocument!.getElementById("inputElement")!.focus();
            });
            await browser.keys([Key.Ctrl, "v"]);

            const pastedText = await browser.execute(() => {
                const frame = document.getElementById(
                    "other-document",
                ) as HTMLIFrameElement;
                const input = frame.contentDocument!.getElementById(
                    "inputElement",
                ) as HTMLInputElement;
                return input.value;
            });
            expect(pastedText).to.equal("texample");
        });
    });
}
