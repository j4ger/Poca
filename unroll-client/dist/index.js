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
export var UnrollState;
(function (UnrollState) {
    UnrollState[UnrollState["Up"] = 0] = "Up";
    UnrollState[UnrollState["Down"] = 1] = "Down";
})(UnrollState || (UnrollState = {}));
export class Unroll {
    constructor(addr) {
        this.addr = addr;
        this.raw = {};
        this.get_queue = {};
        this.state = UnrollState.Down;
        this.identifier = Symbol();
        unroll_effect_callbacks[this.identifier] = {};
    }
    connect() {
        var _a;
        (_a = this.ws) === null || _a === void 0 ? void 0 : _a.close();
        this.ws = new WebSocket(this.addr);
        this.ws.onopen = () => {
            this.state = UnrollState.Up;
            this.ws.onmessage = this.message_handler;
        };
    }
    close() {
        var _a;
        (_a = this.ws) === null || _a === void 0 ? void 0 : _a.close();
        this.state = UnrollState.Down;
    }
    message_handler(event) {
        var _a, _b;
        const message = JSON.parse(event.data);
        switch (message.messageType) {
            case WSMessageType.Get:
                if (this.get_queue[message.key].length > 0) {
                    (_a = this.get_queue[message.key].shift()) === null || _a === void 0 ? void 0 : _a(message.data);
                }
                break;
            case WSMessageType.Set:
                this.raw[message.key] = JSON.parse(message.data);
                (_b = unroll_effect_callbacks[this.identifier][message.key]) === null || _b === void 0 ? void 0 : _b.forEach((callback) => callback());
                break;
            default:
                console.log(message);
        }
    }
    get_data(key) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const message = {
                messageType: WSMessageType.Get,
                key,
            };
            (_a = this.ws) === null || _a === void 0 ? void 0 : _a.send(JSON.stringify(message));
            return new Promise((resolve, _reject) => {
                this.get_queue[key] = this.get_queue[key] || [];
                this.get_queue[key].push(resolve);
            });
        });
    }
    set_data(key, value) {
        var _a;
        return __awaiter(this, void 0, void 0, function* () {
            const message = {
                messageType: WSMessageType.Set,
                key,
                data: value,
            };
            (_a = this.ws) === null || _a === void 0 ? void 0 : _a.send(JSON.stringify(message));
        });
    }
    reactive(key) {
        return __awaiter(this, void 0, void 0, function* () {
            const that = this;
            const value = JSON.parse(yield this.get_data(key));
            that.raw[key] = value;
            unroll_effect_callbacks[that.identifier][key] = [];
            const result = new Proxy(value, {
                get(target, prop) {
                    if (unroll_setting_up_effect) {
                        unroll_effect_callbacks[that.identifier][key].push(unroll_current_callback);
                    }
                    return target[prop];
                },
                set(target, prop, value) {
                    target[prop] = value;
                    that.set_data(key, JSON.stringify(target));
                    unroll_effect_callbacks[that.identifier][key].forEach((callback) => callback());
                    return true;
                },
            });
            return result;
        });
    }
    reactive_with_default(key, initial_value) {
        const that = this;
        that.set_data(key, JSON.stringify(initial_value));
        that.raw[key] = initial_value;
        unroll_effect_callbacks[that.identifier][key] = [];
        const result = new Proxy(initial_value, {
            get(target, prop) {
                if (unroll_setting_up_effect) {
                    unroll_effect_callbacks[that.identifier][key].push(unroll_current_callback);
                }
                return target[prop];
            },
            set(target, prop, value) {
                target[prop] = value;
                that.set_data(key, JSON.stringify(target));
                unroll_effect_callbacks[that.identifier][key].forEach((callback) => callback());
                return true;
            },
        });
        return result;
    }
}
let unroll_setting_up_effect = false;
let unroll_current_callback = () => { };
let unroll_effect_callbacks = {};
export function unroll_effect(inner) {
    unroll_setting_up_effect = true;
    unroll_current_callback = inner;
    inner();
    unroll_setting_up_effect = false;
}