import { MovieMetadata } from "./movie-metadata.js";
import { DataLoadOptions, URLLoadOptions } from "../config/index.js";

import { ReadyState } from "./index.js";

/**
 * Receives the messages sent by movies to a `LocalConnection` name listened from JavaScript.
 *
 * @param methodName Name of the method the movie asked to call.
 * @param args Arguments of the call, converted from AMF0.
 */
export type LocalConnectionHandler = (
    methodName: string,
    args: unknown[],
) => void;

/**
 * Handles the `ExternalInterface.call` of a movie before they are evaluated as JavaScript.
 *
 * @param name Name passed to `ExternalInterface.call`.
 * @param args Arguments of the call.
 * @returns `{ value }` to answer the call with `value`, or `undefined` to let it be evaluated as usual.
 */
export type ExternalInterfaceHandler = (
    name: string,
    args: unknown[],
) => { value: unknown } | undefined;

/**
 * Handles a navigation requested by a movie (navigateToURL, getURL) before Ruffle performs it.
 *
 * @param url Absolute URL to open (after URL rewrite rules).
 * @param target Target window name ("" for the current one, "_blank", "_self", a named window...).
 * @param method "GET" or "POST" when variables are sent, null otherwise.
 * @param variables Variables sent with the request, or null.
 * @returns true when the handler took over the navigation; Ruffle then does nothing.
 */
export type NavigateToUrlHandler = (
    url: string,
    target: string,
    method: string | null,
    variables: Record<string, string> | null,
) => boolean;

/**
 * A node of the display tree mirror: a display object the mirror's filter retains.
 * Bounds are CSS pixels relative to the player's canvas.
 */
export interface MirrorNode {
    /** Identifier of the node, given by the mirror; a new one is given when an object comes back on stage. */
    uid: number;
    /** uid of the nearest emitted ancestor, or `null` at the root. */
    parent: number | null;
    /** Position among all the children of the display parent (render order). */
    index: number;
    /**
     * `interactive` for buttons and containers handling the mouse as a whole (`mouseChildren = false`),
     * `container` for other containers, `text` for text fields.
     */
    kind: "container" | "interactive" | "text" | "other";
    /** Qualified AVM2 class name, e.g. `mx.controls::TextInput`. */
    className: string;
    /** `DisplayObject.name`. */
    name: string | null;
    /** Flex `UIComponent.id`. */
    id: string | null;
    automationName: string | null;
    label: string | null;
    text: string | null;
    toolTip: string | null;
    enabled: boolean | null;
    /** `TextField` editability, or the `editable` property of the component. */
    editable: boolean | null;
    selectedIndex: number | null;
    /** `itemIndex` of a Spark item renderer, `listData.rowIndex` of an mx one. */
    itemIndex: number | null;
    /** Whether the object and all its ancestors are visible. */
    visible: boolean;
    /** Whether the object, or a descendant that is not emitted, has the focus. */
    focused: boolean;
    bounds: { x: number; y: number; width: number; height: number };
}

/**
 * Statistics of a frame of the display tree mirror.
 */
export interface MirrorStats {
    /** Display objects visited. */
    nodesWalked: number;
    /** Display objects retained by the filter, changed or not. */
    nodesEmitted: number;
    /** ActionScript getters called. */
    gettersCalled: number;
    /** Duration of the walk, in microseconds. */
    elapsedUs: number;
    /** Whether the filter's `maxNodes` was reached. */
    truncated: boolean;
}

/**
 * Changes of the display tree mirror since the previous frame.
 */
export interface MirrorFrame {
    frame: number;
    /** When `true`, the receiver must drop everything it knows: `upserts` is the whole mirror. */
    reset: boolean;
    /** New or changed nodes, parents before children. */
    upserts: MirrorNode[];
    /** uids of the nodes that left the mirror. */
    removed: number[];
    events: MirrorEvent[];
    stats: MirrorStats;
}

/**
 * What happened to a node of the mirror since the previous frame.
 */
export type MirrorEvent =
    | { type: "click"; uid: number; button: "left" | "right" | "middle" }
    | { type: "textInput"; uid: number }
    | { type: "focus"; uid: number | null };

/**
 * Which display objects the mirror retains. Omitted fields take their default value.
 */
export interface MirrorFilter {
    /** Buttons and containers handling the mouse as a whole. Default `true`. */
    interactive?: boolean;
    /** Text fields. Default `true`. */
    text?: boolean;
    /** Objects with a Flex `id`, and list item renderers. Default `true`. */
    identified?: boolean;
    /** Objects that are hidden or have a hidden ancestor; otherwise only the root of a hidden subtree is kept. Default `false`. */
    emitHidden?: boolean;
    /** Glob patterns (`*`, `?`) on the qualified class name, e.g. `mx.skins::*`. */
    excludeClasses?: string[];
    /** Glob patterns on the Flex id. */
    excludeIds?: string[];
    /** Glob patterns on the instance name. */
    excludeNames?: string[];
    /** Maximum number of nodes; beyond, `stats.truncated` is set. Default 5000. */
    maxNodes?: number;
}

/**
 * An input event dispatched to the movie by {@link PlayerV1.mirrorInject}.
 * Coordinates are CSS pixels relative to the canvas.
 */
export type MirrorInputEvent =
    | {
          type: "mouseMove" | "mouseDown" | "mouseUp";
          x: number;
          y: number;
          button?: "left" | "right" | "middle";
      }
    | { type: "wheel"; x: number; y: number; deltaY: number }
    /** Typed text, for the focused text field (as typing it would). */
    | { type: "text"; text: string }
    /** Editing keys; `tab` moves the focus. */
    | {
          type: "textControl";
          code: "selectAll" | "backspace" | "delete" | "enter" | "tab";
      };

/**
 * An entry added by the embedding page to Ruffle's context menu.
 */
export interface ContextMenuEntry {
    text: string;
    /** When defined, the entry shows a checkmark column. */
    checked?: boolean;
    /** Default `true`. */
    enabled?: boolean;
    onClick: () => void;
}

export interface PlayerV1 {
    /**
     * Adds a handler for arbitrary "fs commands" from a movie in this player.
     *
     * @remarks
     * If script access is allowed, a movie may communicate to the page through the ActionScript method `fscommand(name, args)`.
     *
     * The exact commands and their arguments are more or less arbitrary and up to the movie.
     * This is an incredibly deprecated way of communicating between Flash and JavaScript,
     * and was deprecated in favor of `ExternalInterface` in Flash Player 8 (2005).
     */
    addFSCommandHandler(
        /**
         * A command handler to receive `fscommand`s.
         *
         * @param command An arbitrary name of a command.
         * @param args An arbitrary argument to the command.
         */
        handler: (command: string, args: string) => void,
    ): void;

    /**
     * Any configuration that should apply to this specific player.
     * This will be defaulted with any global configuration.
     */
    config: URLLoadOptions | DataLoadOptions | object;

    /**
     * The effective config loaded with the last call to `load()`.
     * If no such call has been made, this will be `null`.
     */
    readonly loadedConfig: URLLoadOptions | DataLoadOptions | null;

    /**
     * Indicates the readiness of the playing movie.
     *
     * @returns The `ReadyState` of the player.
     */
    get readyState(): ReadyState;

    /**
     * The metadata of the playing movie (such as movie width and height).
     * These are inherent properties stored in the SWF file and are not affected by runtime changes.
     * For example, `metadata.width` is the width of the SWF file, and not the width of the Ruffle player.
     *
     * @returns The metadata of the movie, or `null` if the movie metadata has not yet loaded.
     */
    get metadata(): MovieMetadata | null;

    /**
     * Reloads the player, as if you called {@link load} with the same config as the last time it was called.
     *
     * If this player has never been loaded, this method will return an error.
     */
    reload(): Promise<void>;

    /**
     * Loads a specified movie into this player.
     *
     * This will replace any existing movie that may be playing.
     *
     * @param options One of the following:
     * - A URL, passed as a string, which will load a URL with default options.
     * - A {@link URLLoadOptions} object, to load a URL with options.
     * - A {@link DataLoadOptions} object, to load data with options.
     * The options, if provided, must only contain values provided for this specific movie.
     * They must not contain any default values, since those would overwrite other configuration
     * settings with a lower priority (e.g. the general RufflePlayer config).
     *
     * The options will be defaulted by the {@link config} field, which itself
     * is defaulted by a global `window.RufflePlayer.config`.
     */
    load(options: string | URLLoadOptions | DataLoadOptions): Promise<void>;

    /**
     * Returns the master volume of the player.
     *
     * The volume is linear and not adapted for logarithmic hearing.
     *
     * @returns The volume. 1.0 is 100% volume.
     */
    get volume(): number;

    /**
     * Sets the master volume of the player.
     *
     * The volume should be linear and not adapted for logarithmic hearing.
     *
     * @param value The volume. 1.0 is 100% volume.
     */
    set volume(value: number);

    /**
     * Checks if this player is allowed to be fullscreen by the browser.
     *
     * @returns True if you may call {@link requestFullscreen}.
     */
    get fullscreenEnabled(): boolean;

    /**
     * Checks if this player is currently fullscreen inside the browser.
     *
     * @returns True if it is fullscreen.
     */
    get isFullscreen(): boolean;

    /**
     * Requests the browser to make this player fullscreen.
     *
     * This is not guaranteed to succeed, please check {@link fullscreenEnabled} first.
     */
    requestFullscreen(): void;

    /**
     * Requests the browser to no longer make this player fullscreen.
     */
    exitFullscreen(): void;

    /**
     * Checks if this movie is suspended.
     *
     * A suspended movie will not execute any frames, scripts or sounds.
     * This movie is considered inactive and will not wake up until resumed.
     * If no movie is loaded, this method will return true.
     *
     * @see {@link suspend} to suspend the player
     * @see {@link resume} to resume the player from suspension
     * @returns `true` if the movie is suspended or does not exist, `false` if the movie is playing
     */
    get suspended(): boolean;

    /**
     * Suspends the movie.
     *
     * A suspended movie will not execute any frames, scripts or sounds.
     * This movie is considered inactive and will not wake up until resumed.
     * If the movie is already suspended or no movie is loaded, this method will do nothing.
     *
     * @see {@link suspended} to check if the player is suspended
     * @see {@link resume} to resume the player from suspension
     */
    suspend(): void;

    /**
     * Resumes the movie from suspension.
     *
     * The movie will now resume executing any frames, scripts and sounds.
     * If the movie is not suspended or no movie is loaded, this method will do nothing.
     *
     * @see {@link suspended} to suspend the player
     * @see {@link suspend} to check if the player is suspended
     */
    resume(): void;

    /**
     * Sets a trace observer on this flash player.
     *
     * The observer will be called, as a function, for each message that the playing movie will "trace" (output).
     *
     * @param observer The observer that will be called for each trace.
     */
    set traceObserver(observer: ((message: string) => void) | null);

    /**
     * Fetches the loaded SWF and downloads it.
     */
    downloadSwf(): Promise<void>;

    /**
     * Show a dismissible message in front of the player.
     *
     * @param message The message shown to the user.
     */
    displayMessage(message: string): void;

    /**
     * Calls an External Interface callback with the given name and arguments.
     *
     * This will call any ActionScript code assigned to the given name.
     * If no such External Interface callback exists with the given name, this method silently fails and returns `undefined`.
     *
     * @param name Name of the callback to call.
     * @param args Any arguments to pass to the callback.
     * @returns Any value returned by the callback.
     */
    callExternalInterface(name: string, ...args: unknown[]): unknown;

    /**
     * Listens on a `LocalConnection` name from JavaScript, as `LocalConnection.connect` would:
     * messages sent by movies of this player to that name are passed to the handler, after the frame.
     *
     * @param name Name of the connection.
     * @param handler Function receiving the method name and the arguments of each message.
     * @returns `false` if the name is already in use in this player.
     */
    localConnectionListen(
        name: string,
        handler: LocalConnectionHandler,
    ): boolean;

    /**
     * Sends a message to a `LocalConnection` of a movie of this player, as `LocalConnection.send` would.
     * It is delivered on the next frame.
     *
     * @param name Name of the connection.
     * @param methodName Name of the method to call.
     * @param args Arguments, converted to AMF0 (strings, numbers, booleans, null, arrays, plain objects).
     * @returns `false` if no movie is loaded.
     */
    localConnectionSend(
        name: string,
        methodName: string,
        ...args: unknown[]
    ): boolean;

    /**
     * Stops listening on a name opened by {@link localConnectionListen}.
     *
     * @param name Name of the connection.
     */
    localConnectionClose(name: string): void;

    /**
     * Handler of the movie's `ExternalInterface.call`, consulted before the call is evaluated as
     * JavaScript (e.g. to emulate a container with its own call conventions). `null` by default.
     */
    externalInterfaceHandler: ExternalInterfaceHandler | null;

    /**
     * Handler of the movie's navigations (navigateToURL, getURL), consulted before Ruffle opens the URL
     * (e.g. to open it in a tab of the embedding page). `null` by default.
     */
    navigateToUrlHandler: NavigateToUrlHandler | null;

    /**
     * Whether the display tree mirror is active. `false` by default (load option `mirror`).
     * Activating it delivers a frame with `reset` set to {@link mirrorHandler} at the next tick.
     */
    mirrorEnabled: boolean;

    /**
     * Receives a frame of the mirror after the ticks where something changed, at most every
     * `minMs` (see {@link mirrorSetIntervals}). `null` by default.
     */
    mirrorHandler: ((frame: MirrorFrame) => void) | null;

    /**
     * Walks the display tree, reading every property again.
     *
     * @returns A full frame of the mirror (`reset` set), also delivered to {@link mirrorHandler};
     * `null` when the mirror is not active.
     */
    mirrorRefresh(): MirrorFrame | null;

    /**
     * Sets which display objects the mirror retains. The next frame has `reset` set.
     *
     * @param filter The filter; omitted fields take their default value.
     * @throws {Error} If the filter is malformed.
     */
    mirrorSetFilter(filter: MirrorFilter): void;

    /**
     * Sets the minimal delay between two frames of the mirror (default 100 ms), and the period
     * of the slow refresh of the properties read from ActionScript (default 1000 ms, 0 = never).
     *
     * @param minMs Minimal delay between two frames, in milliseconds.
     * @param propertiesMs Period of the slow refresh, in milliseconds.
     */
    mirrorSetIntervals(minMs: number, propertiesMs: number): void;

    /**
     * Dispatches an input event to the movie, as the user would. Available even when the mirror
     * is not active. It does not give the focus to the player.
     *
     * @param event The event, in CSS pixels relative to the canvas.
     * @throws {Error} If the event is malformed.
     */
    mirrorInject(event: MirrorInputEvent): void;

    /**
     * Extra entries of Ruffle's context menu, asked for each time the menu opens.
     * They are shown after the movie's entries. `null` by default.
     */
    contextMenuItems: (() => ContextMenuEntry[]) | null;
}
