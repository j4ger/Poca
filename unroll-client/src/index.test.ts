import { Unroll, UnrollState, unroll_effect } from "./index";

test("Instance is initialized with Down state", () => {
  const unroll = new Unroll("localhost:1145");
  expect(unroll.state).toBe(UnrollState.Down);
});

test("Set and Get value", () => {
  const unroll = new Unroll("localhost:1145");
  const handle = unroll.reactive_with_default("client_info", { id: 114514 });
  expect(handle["id"]).toBe(114514);
  handle["id"] = 1919810;
  expect(handle["id"]).toBe(1919810);
});

test("Side effects", () => {
  const unroll = new Unroll("localhost:1145");
  const handle = unroll.reactive_with_default("client_info", { id: 114514 });
  let listener = { modified: false };
  unroll_effect(() => {
    console.log("Client id is now " + handle["id"]);
    listener.modified = true;
  });
  handle["id"] = 1919810;
  expect(listener.modified).toBe(true);
});
