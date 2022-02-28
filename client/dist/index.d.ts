export declare enum ConnectionState {
    Up = 0,
    Down = 1
}
export declare class Poca {
    readonly addr: string;
    private identifier;
    private ws?;
    private raw;
    private work_pool;
    private get_queue;
    state: ConnectionState;
    constructor(addr: string);
    connect(): Promise<void>;
    close(): void;
    private get_data;
    private set_data;
    reactive<T extends Object, K extends keyof T>(key: string): Promise<T>;
    reactive_with_default<T extends Object, K extends keyof T>(key: string, initial_value: T): T;
    emit(key: string): void;
}
export declare function effect(inner: () => void): void;
