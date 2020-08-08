use crate::{config::Config, error::Result, PLUGINS};
use log::*;
use matrix_sdk::{
    self,
    events::{
        room::message::{MessageEventContent, TextMessageEventContent},
        SyncMessageEvent,
    },
    identifiers::UserId,
    Client, ClientConfig, EventEmitter, Session, SyncRoom, SyncSettings,
};
use matrix_sdk_common_macros::async_trait;
use std::convert::TryFrom;
use url::Url;

pub struct EventCallback;

#[async_trait]
impl EventEmitter for EventCallback {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(_room) = room {
            if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
                sender,
                ..
            } = event
            {
                // TODO actual logic
                if sender.localpart() == "mtrnord" && msg_body.contains("!test") {
                    tokio::spawn(async move {
                        info!("got test command");
                        let plugins = PLUGINS.lock().await;
                        info!("got plugin lock");
                        match plugins.call("test") {
                            Ok(_) => info!("called test"),
                            Err(e) => {
                                error!("{}", e);
                            }
                        }
                    });
                }
            }
        }
    }
}

pub async fn login(config: Config) -> Result<()> {
    let client_config = ClientConfig::new().store_path(config.matrix.store_path.clone());
    let homeserver_url = Url::parse(&config.matrix.homeserver_url)?;
    let mut client = Client::new_with_config(homeserver_url, client_config)?;

    client.add_event_emitter(Box::new(EventCallback {})).await;

    let session = Session {
        access_token: config.matrix.access_token,
        user_id: UserId::try_from(config.matrix.username)?,
        device_id: "mx-gitlab".into(),
    };
    client.restore_login(session).await?;
    info!("{}", client.logged_in().await);
    println!("logged in");

    {
        let mut plugins = PLUGINS.lock().await;
        (*plugins).matrix_client = Some(client.clone());
    }
    match client.clone().sync_token().await {
        Some(token) => {
            let sync_settings = SyncSettings::new().token(token);
            //client.clone().sync(sync_settings.clone()).await?;
            client
                .clone()
                .sync_forever(sync_settings, |_| async {})
                .await;
        }
        None => {
            let sync_settings = SyncSettings::new();
            //client.clone().sync(sync_settings.clone()).await?;
            client
                .clone()
                .sync_forever(sync_settings, |_| async {})
                .await;
        }
    }
    println!("syncing");

    Ok(())
}
