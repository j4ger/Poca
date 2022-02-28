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
  message_type: WSMessageType;
  key?: string;
  data?: string;
}

export class Poca {
  private identifier!: symbol;
  private ws?: WebSocket;
  private raw: { [key: string]: any } = {};
  private work_pool: string[] = [];
  private get_queue: {
    [key: string]: ((value: string | PromiseLike<string>) => void)[];
  } = {};
  state: ConnectionState = ConnectionState.Down;

  constructor(readonly addr: string) {
    this.identifier = Symbol();
    effect_callbacks[this.identifier] = {};
  }

  async connect(): Promise<void> {
    let that = this;
    new Promise((resolve) => {
      that.ws?.close();
      that.ws = new WebSocket("ws://" + this.addr);
      that.ws.onopen = () => {
        that.state = ConnectionState.Up;
        that.ws!.onmessage = (event: MessageEvent<any>) => {
          const message: WSMessage = JSON.parse(event.data);
          switch (message.message_type) {
            case WSMessageType.Get:
              if (this.get_queue[message.key!].length > 0) {
                this.get_queue[message.key!].shift()?.(message.data!);
              }
              break;
            case WSMessageType.Set:
              this.raw[message.key!] = JSON.parse(message.data!);
              //only call callbacks if values are different
              //or should I
              effect_callbacks[this.identifier][message.key!]?.forEach(
                (callback) => callback()
              );
              break;
            default:
              console.log("Unimplemented message: " + message);
          }
        };
        that.work_pool.forEach((key) => {
          let message: WSMessage = {
            message_type: WSMessageType.Get,
            key,
          };
          that.ws!.send(JSON.stringify(message));
        });
        that.work_pool = [];
        resolve(undefined);
      };
    });
  }

  close() {
    this.ws?.close();
    this.state = ConnectionState.Down;
  }

  private async get_data(key: string): Promise<string> {
    const message: WSMessage = {
      message_type: WSMessageType.Get,
      key,
    };

    if (this.state == ConnectionState.Up) {
      this.ws?.send(JSON.stringify(message));
    } else {
      this.work_pool.push(key);
    }

    return new Promise((resolve) => {
      this.get_queue[key] = this.get_queue[key] || [];
      this.get_queue[key].push(resolve);
    });
  }

  private async set_data(key: string, value: string) {
    const message: WSMessage = {
      message_type: WSMessageType.Set,
      key,
      data: value,
    };
    this.ws?.send(JSON.stringify(message));
  }

  async reactive<T extends Object, K extends keyof T>(key: string): Promise<T> {
    const that = this;
    const data = await this.get_data(key);
    const value: T = JSON.parse(JSON.parse(data));
    return new Promise((resolve) => {
      that.raw[key] = value;
      effect_callbacks[that.identifier][key] = [];
      const result = new Proxy(value, {
        get(_target, prop) {
          if (setting_up_effect) {
            effect_callbacks[that.identifier][key].push(current_callback);
          }
          return that.raw[key][prop as K];
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
      resolve(result);
    });
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
