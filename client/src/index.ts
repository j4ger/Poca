enum WSMessageType {
  Set = 1,
  Emit = 2,
  Get = 3,
  Error = 4,
}

export enum ConnectionState {
  Up,
  Down,
}

interface WSMessage {
  messageType: WSMessageType;
  key?: string;
  data?: string;
}

export class Poca {
  private identifier!: symbol;
  private ws?: WebSocket;
  private raw: { [key: string]: any } = {};
  private get_queue: {
    [key: string]: ((value: string | PromiseLike<string>) => void)[];
  } = {};
  state: ConnectionState = ConnectionState.Down;

  constructor(readonly addr: string) {
    this.identifier = Symbol();
    effect_callbacks[this.identifier] = {};
  }

  connect() {
    this.ws?.close();
    this.ws = new WebSocket(this.addr);
    this.ws.onopen = () => {
      this.state = ConnectionState.Up;
      this.ws!.onmessage = this.message_handler;
    };
  }

  close() {
    this.ws?.close();
    this.state = ConnectionState.Down;
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
        effect_callbacks[this.identifier][message.key!]?.forEach((callback) =>
          callback()
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
    effect_callbacks[that.identifier][key] = [];
    const result = new Proxy(value, {
      get(target, prop) {
        if (setting_up_effect) {
          effect_callbacks[that.identifier][key].push(current_callback);
        }
        return target[prop as K];
      },
      set(target, prop, value) {
        target[prop as K] = value;
        that.set_data(key, JSON.stringify(target));
        effect_callbacks[that.identifier][key].forEach((callback) =>
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
    that.raw[key] = initial_value;
    effect_callbacks[that.identifier][key] = [];
    const result = new Proxy(initial_value, {
      get(target, prop) {
        if (setting_up_effect) {
          effect_callbacks[that.identifier][key].push(current_callback);
        }
        return target[prop as K];
      },
      set(target, prop, value) {
        target[prop as K] = value;
        that.set_data(key, JSON.stringify(target));
        effect_callbacks[that.identifier][key].forEach((callback) =>
          callback()
        );
        return true;
      },
    });
    return result;
  }
}

let setting_up_effect = false;
let current_callback = () => {};

let effect_callbacks: {
  [key: symbol]: { [innerKey: string]: (() => void)[] };
} = {};

export function effect(inner: () => void) {
  setting_up_effect = true;
  current_callback = inner;
  inner();
  setting_up_effect = false;
}
