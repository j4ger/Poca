var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var WSMessageType;
(function (WSMessageType) {
    WSMessageType[WSMessageType["Set"] = 1] = "Set";
    WSMessageType[WSMessageType["Emit"] = 2] = "Emit";
    WSMessageType[WSMessageType["Get"] = 3] = "Get";
    WSMessageType[WSMessageType["Error"] = 4] = "Error";
})(WSMessageType || (WSMessageType = {}));
export var ConnectionState;
(function (ConnectionState) {
    ConnectionState[ConnectionState["Up"] = 0] = "Up";
    ConnectionState[ConnectionState["Down"] = 1] = "Down";
})(ConnectionState || (ConnectionState = {}));
export class Poca {
    constructor(addr) {
        this.addr = addr;
        this.raw = {};
        this.work_pool = [];
        this.get_queue = {};
        this.state = ConnectionState.Down;
        this.identifier = Symbol();
        effect_callbacks[this.identifier] = {};
    }
    connect() {
        return __awaiter(this, void 0, void 0, function* () {
            let that = this;
            new Promise((resolve) => {
                var _a;
                (_a = that.ws) === null || _a === void 0 ? void 0 : _a.close();
                that.ws = new WebSocket("ws://" + this.addr);
                that.ws.onopen = () => {
                    that.state = ConnectionState.Up;
                    that.ws.onmessage = (event) => {
                        var _a, _b;
                        const message = JSON.parse(event.data);
                        switch (message.message_type) {
                            case WSMessageType.Get:
                                if (this.get_queue[message.key].length > 0) {
                                    (_a = this.get_queue[message.key].shift()) === null || _a === void 0 ? void 0 : _a(message.data);
                                }
                                break;
                            case WSMessageType.Set:
                                this.raw[message.key] = JSON.parse(message.data);
                                //only call callbacks if values are different
                                //or should I
                                (_b = effect_callbacks[this.identifier][message.key]) === null || _b === void 0 ? void 0 : _b.forEach((callback) => callback());
                                break;
                            default:
                                console.log("Unimplemented message: " + message);
                        }
                    };
                    that.work_pool.forEach((key) => {
                        let message = {
                            message_type: WSMessageType.Get,
                            key,
                        };
                        that.ws.send(JSON.stringify(message));
                    });
                    that.work_pool = [];
                    resolve(undefined);
                };
            });
        });
    }
    close() {
        var _a;
        (_a = this.ws) === null || _a === void 0 ? void 0 : _a.close();
        this.state = ConnectionState.Down;
    }
    get_data(key) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const message = {
                message_type: WSMessageType.Get,
                key,
            };
            if (this.state == ConnectionState.Up) {
                (_a = this.ws) === null || _a === void 0 ? void 0 : _a.send(JSON.stringify(message));
            }
            else {
                this.work_pool.push(key);
            }
            return new Promise((resolve) => {
                this.get_queue[key] = this.get_queue[key] || [];
                this.get_queue[key].push(resolve);
            });
        });
    }
    set_data(key, value) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const message = {
                message_type: WSMessageType.Set,
                key,
                data: value,
            };
            (_a = this.ws) === null || _a === void 0 ? void 0 : _a.send(JSON.stringify(message));
        });
    }
    reactive(key) {
        return __awaiter(this, void 0, void 0, function* () {
            const that = this;
            const data = yield this.get_data(key);
            const value = JSON.parse(JSON.parse(data));
            return new Promise((resolve) => {
                that.raw[key] = value;
                effect_callbacks[that.identifier][key] = [];
                const result = new Proxy(value, {
                    get(_target, prop) {
                        if (setting_up_effect) {
                            effect_callbacks[that.identifier][key].push(current_callback);
                        }
                        return that.raw[key][prop];
                    },
                    set(target, prop, value) {
                        target[prop] = value;
                        that.set_data(key, JSON.stringify(target));
                        effect_callbacks[that.identifier][key].forEach((callback) => callback());
                        return true;
                    },
                });
                resolve(result);
            });
        });
    }
    reactive_with_default(key, initial_value) {
        const that = this;
        that.set_data(key, JSON.stringify(initial_value));
        that.raw[key] = initial_value;
        effect_callbacks[that.identifier][key] = [];
        const result = new Proxy(initial_value, {
            get(target, prop) {
                if (setting_up_effect) {
                    effect_callbacks[that.identifier][key].push(current_callback);
                }
                return target[prop];
            },
            set(target, prop, value) {
                target[prop] = value;
                that.set_data(key, JSON.stringify(target));
                effect_callbacks[that.identifier][key].forEach((callback) => callback());
                return true;
            },
        });
        return result;
    }
    emit(key) {
        var _a;
        const message = {
            message_type: WSMessageType.Emit,
            key
        };
        (_a = this.ws) === null || _a === void 0 ? void 0 : _a.send(JSON.stringify(message));
    }
}
let setting_up_effect = false;
let current_callback = () => { };
let effect_callbacks = {};
export function effect(inner) {
    setting_up_effect = true;
    current_callback = inner;
    inner();
    setting_up_effect = false;
}
