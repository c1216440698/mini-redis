use anyhow::Ok;
use bytes::Bytes;
use mini_redis::Command;
use mini_redis::Connection;
use mini_redis::Frame;
use std::collections::HashMap;
use std::hash::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct Db {
    state: Arc<Vec<Mutex<HashMap<String, Bytes>>>>,
}

fn calculate_hash<T: Hash>(t: &T) -> usize {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish() as usize
}

impl Db {
    fn new(num_shards: usize) -> Self {
        let mut db = Vec::with_capacity(num_shards);
        for _ in 0..num_shards {
            db.push(Mutex::new(HashMap::new()));
        }

        Self {
            state: Arc::new(db),
        }
    }

    async fn handle_connection(self, socket: TcpStream) -> anyhow::Result<()> {
        let mut connection = Connection::new(socket);
        while let Some(frame) = connection
            .read_frame()
            .await
            .map_err(|e| anyhow::anyhow!(e))?
        {
            let cmd = Command::from_frame(frame).map_err(|e| anyhow::anyhow!(e))?;
            println!("{:?}", cmd);
            let resp = self.apply_command(cmd).await?;
            connection.write_frame(&resp).await?;
        }

        Ok(())
    }

    async fn apply_command(&self, cmd: Command) -> anyhow::Result<Frame> {
        let resp = match cmd {
            Command::Get(cmd) => {
                let hash = calculate_hash(&cmd.key());
                let shard = self
                    .state
                    .get(hash % self.state.len())
                    .expect("mod operator guarntee it must exist");
                if let Some(v) = shard.lock().await.get(cmd.key()) {
                    Ok(Frame::Bulk(v.clone()))
                } else {
                    Ok(Frame::Null)
                }
            }
            Command::Set(cmd) => {
                let hash = calculate_hash(&cmd.key());
                let shard = self
                    .state
                    .get(hash % self.state.len())
                    .expect("mod operator guarntee it must exist");
                shard
                    .lock()
                    .await
                    .insert(cmd.key().to_string(), cmd.value().clone());
                Ok(Frame::Simple("OK".to_string()))
            }
            Command::Publish(cmd) => {
                todo!()
            }
            Command::Subscribe(cmd) => {
                todo!()
            }
            Command::Unsubscribe(cmd) => {
                todo!()
            }
            Command::Unknown(cmd) => {
                panic!("unkown command");
            }
        };
        return resp;
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const NUM_SHARDS: usize = 8;
    let listener = TcpListener::bind("127.0.0.1:7878").await?;
    let shared_state = Db::new(NUM_SHARDS);
    loop {
        let (stream, addr) = listener.accept().await?;
        let shared_state_clone = shared_state.clone();
        tokio::spawn(async move {
            if let Err(e) = shared_state_clone.handle_connection(stream).await {
                println!("handle socket addr: {} failed, reason: {}", addr, e);
            }
        });
    }
}
