use std::io::{self, BufRead, Write};

use anyhow::Result;
use grammers_client::{Client, grammers_tl_types as tl, types::PackedChat};

use crate::defs::{BOT_SESSION_FILE, SESSION_FILE};

pub fn prompt(message: &str) -> Result<String> {
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(message.as_bytes())?;
    stdout.flush()?;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

pub async fn get_common_chats(
    client: &Client,
    user: &PackedChat,
    per_page_limit: usize,
) -> Result<Vec<tl::enums::Chat>> {
    let mut chats = Vec::new();
    let mut max_id = 0_i64;

    loop {
        let req = tl::functions::messages::GetCommonChats {
            user_id: tl::enums::InputUser::User(tl::types::InputUser {
                user_id: user.id,
                access_hash: user.access_hash.unwrap(),
            }),
            max_id,
            limit: per_page_limit as i32,
        };
        let resp = match client.invoke(&req).await {
            Ok(s) => s,
            Err(e) => {
                log::error!(
                    "get common chats failed: {}, hash: {:?}",
                    e,
                    user.access_hash
                );
                break;
            }
        };
        let slice = resp.chats();
        if slice.is_empty() {
            continue;
        }
        chats.extend(slice.clone());
        if slice.clone().len() < per_page_limit {
            break;
        }
        if let Some(last) = slice.last() {
            max_id = last.id();
        }
    }

    Ok(chats)
}

pub async fn get_access_hash(client: &Client, target_id: i64) -> Result<Option<i64>> {
    let input_user = tl::enums::InputUser::User(tl::types::InputUser {
        user_id: target_id,
        access_hash: 0,
    });

    let resp = client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![input_user],
        })
        .await?;

    if let Some(tl::enums::User::User(u)) = resp.into_iter().next() {
        Ok(u.access_hash)
    } else {
        Ok(None)
    }
}

pub fn save_session(client: &Client, bot: &Client) {
    match client.session().save_to_file(SESSION_FILE) {
        Ok(_) => {}
        Err(e) => {
            log::error!("NOTE: failed to save the session, will sign out when done: {e}");
        }
    }
    match bot.session().save_to_file(BOT_SESSION_FILE) {
        Ok(_) => {}
        Err(e) => {
            log::error!("NOTE: failed to save the session, will sign out when done: {e}");
        }
    }
}
