enum WSMessageType {
  Set = 1,
  Emit = 2,
  Get = 3,
  Error = 4,
}

export enum UnrollState {
  Up,
  Down,
}

interface WSMessage {
  messageType: WSMessageType;
  key?: string;
  data?: string;
}

export class Unroll {
  private identifier!: symbol;
  private ws?: WebSocket;
  private raw: { [key: string]: any } = {};
  private get_queue: {
    [key: string]: ((value: string | PromiseLike<string>) => void)[];
  } = {};
  state: UnrollState = UnrollState.Down;

  constructor(readonly addr: string) {
    this.identifier = Symbol();
  }

  connect() {
    this.ws?.close();
    this.ws = new WebSocket(this.addr);
    this.ws.onopen = () => {
      this.state = UnrollState.Up;
      this.ws!.onmessage = this.message_handler;
    };
  }

  close() {
    this.ws?.close();
    this.state = UnrollState.Down;
  }

  private message_handler(event: MessageEvent<any>) {
    const message: WSMessage = JSON.parse(event.data);
    switch (message.messageType) {
      case WSMessageType.Get:
        if (this.get_queue[message.key!].length > 0) {
          this.get_queue[message.key!].shift()?.(message.data!);
        }
        break;
      case WSMessageType.Set:
        this.raw[message.key!] = JSON.parse(message.data!);
        unroll_effect_callbacks[this.identifier][message.key!]?.forEach(
          (callback) => callback()
        );
        break;
      default:
        console.log(message);
    }
  }

  private async get_data(key: string): Promise<string> {
    const message: WSMessage = {
      messageType: WSMessageType.Get,
      key,
    };

    this.ws?.send(JSON.stringify(message));

    return new Promise<string>((resolve, _reject) => {
      this.get_queue[key] = this.get_queue[key] || [];
      this.get_queue[key].push(resolve);
    });
  }

  private async set_data(key: string, value: string) {
    const message: WSMessage = {
      messageType: WSMessageType.Set,
      key,
      data: value,
    };
    this.ws?.send(JSON.stringify(message));
  }

  async reactive<T extends Object, K extends keyof T>(key: string): Promise<T> {
    const that = this;
    const value: T = JSON.parse(await this.get_data(key));
    that.raw[key] = value;
    const result = new Proxy(value, {
      get(target) {
        if (unroll_setting_up_effect) {
          unroll_effect_callbacks[that.identifier][key].push(
            unroll_current_callback
          );
        }
        return () => target;
      },
      set(target, prop, value) {
        target[prop as K] = value;
        that.set_data(key, JSON.stringify(target));
        unroll_effect_callbacks[that.identifier][key].forEach((callback) =>
          callback()
        );
        return true;
      },
    });
    return result;
  }

  reactive_with_default<T extends Object, K extends keyof T>(
    key: string,
    initial_value: T
  ): T {
    const that = this;
    that.set_data(key, JSON.stringify(initial_value));
    const result = new Proxy(initial_value, {
      get(target) {
        if (unroll_setting_up_effect) {
          unroll_effect_callbacks[that.identifier][key].push(
            unroll_current_callback
          );
        }
        return () => target;
      },
      set(target, prop, value) {
        target[prop as K] = value;
        that.set_data(key, JSON.stringify(target));
        unroll_effect_callbacks[that.identifier][key].forEach((callback) =>
          callback()
        );
        return true;
      },
    });
    return result;
  }
}

let unroll_setting_up_effect = false;
let unroll_current_callback = () => {};

let unroll_effect_callbacks: {
  [key: symbol]: { [innerKey: string]: (() => void)[] };
} = {};

export function unroll_effect(inner: () => void) {
  unroll_setting_up_effect = true;
  unroll_current_callback = inner;
  inner();
  unroll_setting_up_effect = false;
}
