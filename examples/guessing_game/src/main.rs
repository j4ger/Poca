use poca::{include_app_dir, DataHandle, Poca};
use rand::Rng;

lazy_static::lazy_static! {
    static ref POCA: Poca = Poca::new("localhost:2341", include_app_dir!("frontend/dist/"));
    static ref GUESS: DataHandle<Guess> = POCA.data("guess", Guess{ guess:"101".to_string()});
    static ref ANSWER: DataHandle<Answer> = POCA.data("answer", Answer { answer: "Input something!".to_string()});
}

use ts2rs::import;

import!("frontend/src/interface.ts");

#[tokio::main]
async fn main() {
    lazy_static::initialize(&ANSWER);
    let target = rand::thread_rng().gen_range(0..100);
    GUESS.on_change(move |new_guess| match new_guess.guess.parse::<i32>() {
        Ok(guess) => {
            if guess == target {
                ANSWER.set(Answer {
                    answer: "You win!".to_string(),
                });
            } else if guess > target {
                ANSWER.set(Answer {
                    answer: "Too high!".to_string(),
                });
            } else {
                ANSWER.set(Answer {
                    answer: "Too low!".to_string(),
                });
            }
        }
        Err(_) => {
            ANSWER.set(Answer {
                answer: "Input should be a number!".to_string(),
            });
        }
    });
    POCA.start().await;
    tokio::signal::ctrl_c().await.unwrap();
}
