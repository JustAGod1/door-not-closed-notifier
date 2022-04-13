use std::collections::HashSet;
use futures::StreamExt;
use telegram_bot::*;
use redis::{Client as RedisClient, Commands};

struct State {
    api: Api,
    inside: bool,
}

#[tokio::main]
async fn main() {
    let token = std::env::var("TOKEN").unwrap();

    let mut state = State {
        api: Api::new(token),
        inside: false,
    };


    loop {
        let result = run(&mut state).await;

        if let Err(e) = result {
            eprintln!("{}", e)
        }
    }
}

const KEY: &str = "mac_addresses";
const TARGET: &str = "f0:a3:5a:1b:ba:ba";
const ID: i64 = 504208153;

async fn run(state: &mut State) -> Result<(), String> {
    let redis = RedisClient::open("redis://127.0.0.1/")
        .map(|a| a.get_connection())
        .map_err(|a| format!("Cannot connect to redis {}", a.to_string()))?;
    let mut redis = redis
        .map_err(|a| format!("Cannot connect to redis {}", a.to_string()))?;

    let members: HashSet<String> = redis.smembers(KEY).map_err(|a| a.to_string())?;

    if state.inside {
        if !members.contains(TARGET) {
            state.api.send(
                SendMessage::new(
                    ChatRef::from_chat_id(ChatId::new(ID)),
                    "Любимая, пока пока, но будь добра, пожалуйста, не забудь закрыть дверь. Заранее спасибо.",
                )
            ).await.map_err(|a| a.to_string())?;
            state.inside = false;
        }
    } else {
        if members.contains(TARGET) {
            state.api.send(
                SendMessage::new(
                    ChatRef::from_chat_id(ChatId::new(ID)),
                    "С возвращением, хорошкинс. Не забудь закрыть дверь пожалуйста.",
                )
            ).await.map_err(|a| a.to_string())?;
            state.inside = true;
        }
    }


    // Fetch new updates via long poll method
    let mut stream = state.api.stream();
    for _ in 0..5 {
        if let Some(update) = stream.next().await {
            // If the received update contains a new message...
            let update = update.map_err(|a| a.to_string())?;
            if let UpdateKind::Message(message) = update.kind {
                if let MessageKind::Text { ref data, .. } = message.kind {
                    // Print received text message to stdout.
                    println!("<{}>: {}", &message.from.first_name, data);

                    // Answer message with "Hi".
                    state.api.send(message.text_reply(format!(
                        "Hi, {}! You just wrote '{}'",
                        &message.from.id, data
                    ))).await.map_err(|a| a.to_string())?;
                }
            }
        }
    }

    Ok(())
}