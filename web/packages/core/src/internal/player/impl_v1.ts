import { PlayerV1, ReadyState } from "../../public/player/index.js";
import { InnerPlayer } from "./inner.js";
import type {
    DataLoadOptions,
    URLLoadOptions,
} from "../../public/config/index.js";
import type {
    ExternalInterfaceHandler,
    LocalConnectionHandler,
    MovieMetadata,
    NavigateToUrlHandler,
} from "../../public/player/index.js";

export class PlayerV1Impl implements PlayerV1 {
    #inner: InnerPlayer;

    constructor(inner: InnerPlayer) {
        this.#inner = inner;
    }

    addFSCommandHandler(handler: (command: string, args: string) => void) {
        this.#inner.addFSCommandHandler(handler);
    }

    get readyState(): ReadyState {
        return this.#inner._readyState;
    }

    get metadata(): MovieMetadata | null {
        return this.#inner.metadata;
    }

    get loadedConfig(): URLLoadOptions | DataLoadOptions | null {
        return this.#inner.loadedConfig ?? null;
    }

    async reload(): Promise<void> {
        await this.#inner.reload();
    }

    async load(
        options: string | URLLoadOptions | DataLoadOptions,
        isPolyfillElement: boolean = false,
    ): Promise<void> {
        await this.#inner.load(options, isPolyfillElement);
    }

    resume(): void {
        this.#inner.play();
    }

    get isPlaying(): boolean {
        return this.#inner.isPlaying;
    }

    get volume(): number {
        return this.#inner.volume;
    }

    set volume(value: number) {
        this.#inner.volume = value;
    }

    get fullscreenEnabled(): boolean {
        return this.#inner.fullscreenEnabled;
    }

    get isFullscreen(): boolean {
        return this.#inner.isFullscreen;
    }

    setFullscreen(isFull: boolean): void {
        this.#inner.setFullscreen(isFull);
    }

    requestFullscreen(): void {
        this.#inner.enterFullscreen();
    }

    exitFullscreen(): void {
        this.#inner.exitFullscreen();
    }

    async downloadSwf(): Promise<void> {
        await this.#inner.downloadSwf();
    }

    displayMessage(message: string): void {
        this.#inner.displayMessage(message);
    }

    suspend(): void {
        this.#inner.pause();
    }

    get suspended(): boolean {
        return !this.#inner.isPlaying;
    }

    set traceObserver(observer: ((message: string) => void) | null) {
        this.#inner.traceObserver = observer;
    }

    get config(): URLLoadOptions | DataLoadOptions | object {
        return this.#inner.config;
    }

    set config(value: URLLoadOptions | DataLoadOptions | object) {
        this.#inner.config = value;
    }

    callExternalInterface(name: string, ...args: unknown[]): unknown {
        return this.#inner.callExternalInterface(name, args);
    }

    localConnectionListen(
        name: string,
        handler: LocalConnectionHandler,
    ): boolean {
        return this.#inner.localConnectionListen(name, handler);
    }

    localConnectionSend(
        name: string,
        methodName: string,
        ...args: unknown[]
    ): boolean {
        return this.#inner.localConnectionSend(name, methodName, args);
    }

    localConnectionClose(name: string): void {
        this.#inner.localConnectionClose(name);
    }

    get navigateToUrlHandler(): NavigateToUrlHandler | null {
        return this.#inner.navigateToUrlHandler;
    }

    set navigateToUrlHandler(handler: NavigateToUrlHandler | null) {
        this.#inner.navigateToUrlHandler = handler;
    }

    get externalInterfaceHandler(): ExternalInterfaceHandler | null {
        return this.#inner.externalInterfaceHandler;
    }

    set externalInterfaceHandler(handler: ExternalInterfaceHandler | null) {
        this.#inner.externalInterfaceHandler = handler;
    }
}
