use std::io::Write;

use anyhow::Result;
use env_logger::Builder;
use grammers_client::{
    Client, Config, InitParams, InputMessage, ReconnectionPolicy, SignInError, Update,
    grammers_tl_types::{self as tl},
    session::Session,
    types::PackedChat,
};

use crate::{
    config::MainConfig,
    defs::{API_HASH, API_ID, BOT_SESSION_FILE, SESSION_FILE, SUPER_ADMIN},
    utils::{get_access_hash, get_common_chats, prompt, save_session},
};

mod config;
mod defs;
mod utils;

struct Reconnection;

impl ReconnectionPolicy for Reconnection {
    fn should_retry(&self, attempts: usize) -> std::ops::ControlFlow<(), std::time::Duration> {
        if attempts > 5 {
            panic!("reconnect attempts is 5, but not connect");
        }
        let duration = u64::pow(5, attempts as _);
        std::ops::ControlFlow::Continue(std::time::Duration::from_millis(duration))
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    let mut builder = Builder::new();

    builder.format(|buf, record| {
        let local_time = chrono::Local::now();
        let time_str = local_time.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

        writeln!(
            buf,
            "[{}] [{}] {} {}",
            time_str,
            record.level(),
            record.target(),
            record.args()
        )
    });
    builder.filter_level(log::LevelFilter::Info).init();
    log::info!("Connecting to Telegram...");
    let client = Client::connect(Config {
        session: Session::load_file_or_create(SESSION_FILE)?,
        api_id: API_ID,
        api_hash: API_HASH.to_string(),
        params: InitParams {
            reconnection_policy: &Reconnection,
            ..Default::default()
        },
    })
    .await?;
    let bot = Client::connect(Config {
        session: Session::load_file_or_create(BOT_SESSION_FILE)?,
        api_id: API_ID,
        api_hash: API_HASH.to_string(),
        params: InitParams {
            reconnection_policy: &Reconnection,
            ..Default::default()
        },
    })
    .await?;
    log::info!("Connected!");

    if !client.is_authorized().await? {
        println!("Signing in...");
        let phone = prompt("Enter your phone number (international format): ")?;
        let token = client.request_login_code(&phone).await?;
        let code = prompt("Enter the code you received: ")?;
        let signed_in = client.sign_in(&token, &code).await;
        match signed_in {
            Err(SignInError::PasswordRequired(password_token)) => {
                let hint = password_token.hint().unwrap_or("None");
                let prompt_message = format!("Enter the password (hint {}): ", &hint);
                let password = prompt(prompt_message.as_str())?;

                client
                    .check_password(password_token, password.trim())
                    .await?;
            }
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        };
        println!("Signed in!");
    }

    if !bot.is_authorized().await? {
        let token = prompt("bot token:")?;
        bot.bot_sign_in(&token).await?;
        println!("Signed in!");
    }

    save_session(&client, &bot);

    MainConfig::init();
    loop {
        let update = bot.next_update().await?;
        match update {
            Update::NewMessage(msg) => {
                let text = msg.text();
                let chat = msg.chat();
                let reply_id = Some(msg.clone().raw.id);

                if text.starts_with("/addadmin")
                    && chat.pack().is_user()
                    && chat.id() == SUPER_ADMIN
                {
                    let mut config = MainConfig::read_config();
                    let mut admins = config.admins.clone();
                    let user = text.trim_start_matches("/addadmin").trim();
                    let admin = user.parse::<i64>().unwrap();
                    admins.insert(admin);
                    config.admins = admins;
                    MainConfig::rewrite_config(Some(config));
                    bot.send_message(
                        chat.clone(),
                        InputMessage::text("已添加").reply_to(reply_id),
                    )
                    .await?;
                }
                if text.starts_with("/addgroup")
                    && chat.pack().is_user()
                    && chat.id() == SUPER_ADMIN
                {
                    let mut config = MainConfig::read_config();
                    let mut groups = config.groups.clone();
                    let user = text.trim_start_matches("/addgroup").trim();
                    let group = user.parse::<i64>().unwrap();
                    groups.insert(group);
                    config.groups = groups;
                    MainConfig::rewrite_config(Some(config));
                    bot.send_message(
                        chat.clone(),
                        InputMessage::text("已添加").reply_to(reply_id),
                    )
                    .await?;
                }
                if text.starts_with("/c ") {
                    let config = MainConfig::read_config();
                    let group = config.groups;
                    let admins = config.admins;
                    let user = text.trim_start_matches("/c").trim();
                    let sended_msg = bot
                        .send_message(
                            chat.clone(),
                            InputMessage::text("查询中...").reply_to(reply_id),
                        )
                        .await?;
                    if user.is_empty() {
                        bot.edit_message(chat, sended_msg.id(), "目标不能为空")
                            .await?;
                        continue;
                    }
                    #[allow(unused_assignments)]
                    let mut packed_user = PackedChat {
                        ty: grammers_client::session::PackedType::User,
                        id: 0,
                        access_hash: Some(0),
                    };

                    if user.starts_with("@") {
                        let u = match client
                            .resolve_username(user.trim_start_matches("@").trim())
                            .await?
                        {
                            Some(s) => s,
                            None => {
                                bot.edit_message(chat, sended_msg.id(), "未知用户").await?;
                                continue;
                            }
                        };
                        packed_user = u.pack();
                    } else {
                        let id = match user.parse::<i64>() {
                            Ok(i) => i,
                            Err(_) => {
                                bot.edit_message(chat, sended_msg.id(), "id解析失败")
                                    .await?;
                                continue;
                            }
                        };
                        #[allow(unused_assignments)]
                        let mut hash = Some(0);
                        hash = get_access_hash(&client, id).await?;

                        if matches!(hash, Some(0) | None) {
                            log::error!(
                                "get user access_hash using getusers failed: user: {}, hash: {:?}",
                                id,
                                hash
                            );
                            bot.edit_message(
                                chat.clone(),
                                sended_msg.id(),
                                "不能获取用户access_hash，你能确保我见过吗",
                            )
                            .await?;
                            continue;
                        }

                        packed_user = PackedChat {
                            ty: grammers_client::session::PackedType::User,
                            id,
                            access_hash: hash,
                        };
                    }

                    if packed_user.id == client.get_me().await?.id()
                        || packed_user.id == bot.get_me().await?.id()
                    {
                        bot.edit_message(chat, sended_msg.id(), "不能查询自身")
                            .await?;
                        continue;
                    }
                    let mut count = 0;
                    let mut admin_list = Vec::new();
                    let groups = get_common_chats(&client, &packed_user, 100).await?;
                    for i in groups {
                        if group.contains(&i.id()) {
                            let mut title = String::new();
                            let mut username = String::new();
                            match i {
                                tl::enums::Chat::Channel(c) => {
                                    title = c.title;
                                    if let Some(s) = c.username {
                                        username = format!("@{}", s);
                                    } else {
                                        username = "N/A".to_string();
                                    }
                                }
                                _ => {}
                            }
                            admin_list.insert(count, (title, username));
                            count += 1;
                        }
                    }

                    if count == 0 {
                        bot.edit_message(chat.clone(), sended_msg.id(), "未查询到共同群")
                            .await?;
                        continue;
                    }
                    let send_msg = {
                        if admins.contains(&chat.id()) && msg.chat().pack().is_user() {
                            let s: String = admin_list
                                .iter()
                                .map(|(t, u)| format!("{} - {}", t, u))
                                .collect::<Vec<_>>()
                                .join("\n");

                            format!(
                                r#"与用户 {} 共同群 {} 个
具体群组:
{}"#,
                                packed_user.id.to_string(),
                                count,
                                s
                            )
                        } else {
                            format!("与用户 {} 共同群 {} 个", packed_user.id.to_string(), count)
                        }
                    };
                    bot.edit_message(chat.clone(), sended_msg.id(), send_msg)
                        .await?;
                }
                if text == "?c" {
                    let config = MainConfig::read_config();
                    let group = config.groups;
                    let admins = config.admins;
                    let reply = match msg.get_reply().await? {
                        Some(r) => r,
                        None => {
                            bot.send_message(
                                chat.clone(),
                                InputMessage::text("请回复消息").reply_to(reply_id),
                            )
                            .await?;
                            continue;
                        }
                    };
                    let sender = match reply.sender() {
                        Some(s) => s,
                        None => {
                            bot.send_message(
                                chat.clone(),
                                InputMessage::text("无法获取发送者").reply_to(reply_id),
                            )
                            .await?;
                            continue;
                        }
                    };
                    let sender = sender.pack();
                    if let None = sender.access_hash {
                        bot.send_message(
                            chat.clone(),
                            InputMessage::text("无法获取access_hash").reply_to(reply_id),
                        )
                        .await?;
                        continue;
                    }
                    if sender.id == client.get_me().await?.id()
                        || sender.id == bot.get_me().await?.id()
                    {
                        bot.send_message(
                            chat.clone(),
                            InputMessage::text("不能查询自身").reply_to(reply_id),
                        )
                        .await?;
                        continue;
                    }
                    let sended_msg = bot
                        .send_message(
                            chat.clone(),
                            InputMessage::text("查询中...").reply_to(reply_id),
                        )
                        .await?;
                    let mut count = 0;
                    let mut admin_list = Vec::new();
                    let groups = get_common_chats(&client, &sender, 100).await?;
                    for i in groups {
                        if group.contains(&i.id()) {
                            let mut title = String::new();
                            let mut username = String::new();
                            match i {
                                tl::enums::Chat::Channel(c) => {
                                    title = c.title;
                                    if let Some(s) = c.username {
                                        username = format!("@{}", s);
                                    } else {
                                        username = "N/A".to_string();
                                    }
                                }
                                _ => {}
                            }
                            admin_list.insert(count, (title, username));
                            count += 1;
                        }
                    }

                    if count == 0 {
                        bot.edit_message(chat.clone(), sended_msg.id(), "未查询到共同群")
                            .await?;
                        continue;
                    }
                    let send_msg = {
                        if admins.contains(&chat.id()) && msg.chat().pack().is_user() {
                            let s: String = admin_list
                                .iter()
                                .map(|(t, u)| format!("{} - {}", t, u))
                                .collect::<Vec<_>>()
                                .join("\n");

                            format!(
                                r#"与用户 {} 共同群 {} 个
具体群组:
{}"#,
                                sender.id.to_string(),
                                count,
                                s
                            )
                        } else {
                            format!("与用户 {} 共同群 {} 个", sender.id.to_string(), count)
                        }
                    };
                    bot.edit_message(chat.clone(), sended_msg.id(), send_msg)
                        .await?;
                }
            }
            _ => {}
        }
    }
}
