use crate::error::Result;
use crate::PLUGINS;
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
use std::path::Path;
use url::Url;

pub struct EventCallback {
    client: Client,
}

#[async_trait]
impl EventEmitter for EventCallback {
    async fn on_room_message(&self, room: SyncRoom, event: &SyncMessageEvent<MessageEventContent>) {
        if let SyncRoom::Joined(room) = room {
            if let SyncMessageEvent {
                content: MessageEventContent::Text(TextMessageEventContent { body: msg_body, .. }),
                sender,
                ..
            } = event
            {
                // TODO actual logic
                if sender.localpart() == "mtrnord" && msg_body.contains("!gitlab_test") {
                    PLUGINS
                        .lock()
                        .expect("Could not lock mutex")
                        .call("gitlab_print_username");
                    //let result = self.plugins.call("gitlab_print_username", &[""]).unwrap();
                    //let room_locked = room.read().await;
                    //self.client.room_send(&room_locked.room_id, result, None).await.unwrap();
                }
            }
        }
    }
}

pub async fn login<P: AsRef<Path>>(
    homeserver_url: String,
    username: String,
    access_token: String,
    store_path: P,
) -> Result<()> {
    let client_config = ClientConfig::new().store_path(store_path);
    let homeserver_url = Url::parse(&homeserver_url)?;
    let mut client = Client::new_with_config(homeserver_url, client_config)?;

    client
        .add_event_emitter(Box::new(EventCallback {
            client: client.clone(),
        }))
        .await;

    let session = Session {
        access_token,
        user_id: UserId::try_from(username)?,
        device_id: "mx-gitlab".to_string(),
    };
    client.restore_login(session).await?;

    println!("logged in");
    match client.sync_token().await {
        Some(token) => {
            let sync_settings = SyncSettings::new().token(token);
            client.sync(sync_settings.clone()).await?;
            client.sync_forever(sync_settings, |_| async {}).await;
        }
        None => {
            let sync_settings = SyncSettings::new();
            client.sync(sync_settings.clone()).await?;
            client.sync_forever(sync_settings, |_| async {}).await;
        }
    }
    println!("syncing");

    Ok(())
}
