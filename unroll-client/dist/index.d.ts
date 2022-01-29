export declare enum UnrollState {
    Up = 0,
    Down = 1
}
export declare class Unroll {
    readonly addr: string;
    private identifier;
    private ws?;
    private raw;
    private get_queue;
    state: UnrollState;
    constructor(addr: string);
    connect(): void;
    close(): void;
    private message_handler;
    private get_data;
    private set_data;
    reactive<T extends Object, K extends keyof T>(key: string): Promise<T>;
    reactive_with_default<T extends Object, K extends keyof T>(key: string, initial_value: T): T;
}
export declare function unroll_effect(inner: () => void): void;
