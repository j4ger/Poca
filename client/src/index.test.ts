import { Poca, ConnectionState, effect } from "./index";

test("Instance is initialized with Down state", () => {
  const poca = new Poca("localhost:1145");
  expect(poca.state).toBe(ConnectionState.Down);
});

test("Set and Get value", () => {
  const poca = new Poca("localhost:1145");
  const handle = poca.reactive_with_default("client_info", { id: 114514 });
  expect(handle["id"]).toBe(114514);
  handle["id"] = 1919810;
  expect(handle["id"]).toBe(1919810);
});

test("Side effects", () => {
  const poca = new Poca("localhost:1145");
  const handle = poca.reactive_with_default("client_info", { id: 114514 });
  let listener = { modified: false };
  effect(() => {
    console.log("Client id is now " + handle["id"]);
    listener.modified = true;
  });
  handle["id"] = 1919810;
  expect(listener.modified).toBe(true);
});
