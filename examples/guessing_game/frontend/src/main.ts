import "./style.css";

import { Poca, effect } from "../../../../client/dist/index";
import { Guess, Answer } from "./interface";

const POCA = new Poca("localhost:2341");
await POCA.connect();

const GUESS: Guess = await POCA.reactive("guess");
const ANSWER: Answer = await POCA.reactive("answer");

const input_field = document.querySelector("#input") as HTMLInputElement;
input_field.addEventListener("input", () => {
  GUESS.guess = input_field.value;
});

effect(() => {
  const result_field = document.querySelector("#result") as HTMLElement;
  console.log(ANSWER.answer);
  result_field.innerHTML = ANSWER.answer;
});
