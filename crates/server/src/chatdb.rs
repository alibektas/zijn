use chrono::NaiveDateTime;
use futures_util::Stream;
use mongodb::{
    Client,
    bson::{Bson, Document, doc},
    options::{ClientOptions, FindOptions},
};
use uuid::Uuid;

struct ChatDB {
    client: Client,
}

#[derive(Debug)]
struct ChatData {
    user_id: Uuid,
    chat_id: Uuid,
    question: String,
    answer: String,
    timestamp: NaiveDateTime,
}

#[derive(Debug)]
enum ChatDBError {
    Mongo(mongodb::error::Error),
    Uuid(uuid::Error),
    BsonFormat,
}

impl From<mongodb::error::Error> for ChatDBError {
    fn from(err: mongodb::error::Error) -> Self {
        ChatDBError::Mongo(err)
    }
}

impl From<uuid::Error> for ChatDBError {
    fn from(err: uuid::Error) -> Self {
        ChatDBError::Uuid(err)
    }
}

impl ChatDB {
    async fn new() -> Self {
        let client_uri = "mongodb://root:example@localhost:27017";
        let options = ClientOptions::parse(client_uri)
            .await
            .expect("Invalid MongoDB options");
        let client = Client::with_options(options).expect("Failed to connect");

        Self { client }
    }

    /// Get all chats sorted by timestamps in ascending order
    async fn get_all_chats(&self, user_id: Uuid) -> Result<Vec<ChatData>, ChatDBError> {
        let db = self.client.database("mydb");
        let collection = db.collection::<Document>("user_data");

        let filter = doc! { "user_id": user_id.to_string() };
        let doc_opt = collection.find_one(filter).await?;

        if let Some(doc) = doc_opt {
            if let Some(data) = doc.get_document("data").ok() {
                for (uuid_str, bson_val) in data.iter() {
                    let uuid = Uuid::parse_str(uuid_str)?;
                    if let Bson::Document(chat_doc) = bson_val {
                        let question = chat_doc.get_str("question").ok().map(str::to_string);
                        let answer = chat_doc.get_str("answer").ok().map(str::to_string);

                        if let (Some(question), Some(answer)) = (question, answer) {
                            history.insert(
                                uuid,
                                ChatData {
                                    question,
                                    answer,
                                    user_id,
                                    chat_id: todo!(),
                                    timestamp: todo!(),
                                },
                            );
                        } else {
                            return Err(ChatDBError::BsonFormat);
                        }
                    }
                }
            }
        }

        Ok(history)
    }

    pub async fn get_all_(
        &self,
        user_id: Uuid,
    ) -> Result<impl Stream<Item = Result<ChatData, ChatDBError>>, ChatDBError> {
        let db = self.client.database("mydb");
        let collection = db.collection::<ChatData>("chats");

        let filter = doc! { "user_id": user_id.to_string() };
        let options = FindOptions::builder().sort(doc! { "timestamp": 1 }).build();

        let cursor = collection.find(filter).with_options(options).await?;
        todo!()
    }

    async fn get_recent_chats(&self, user_id: Uuid, until: NaiveDateTime) {}
}
