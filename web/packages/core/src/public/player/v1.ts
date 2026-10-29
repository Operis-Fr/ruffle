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
}
