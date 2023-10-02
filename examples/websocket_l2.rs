use coinbase_pro_rs::{structs::wsfeed::*, WSFeed, WS_SANDBOX_URL};
use futures::{StreamExt, TryStreamExt};

static KEY: &str = "1d0dc0f7b4e808d430b95d8fed7df3ea";
static SECRET: &str =
    "dTUic8DZPqkS77vxhJFEX5IBr13FcFHTzWYOARgT9kDWGdN03uvxBbH/hVy8f4O5RDmuf+9wNpEfhYhw2FCWyA==";
static PASSPHRASE: &str = "sandbox";

#[tokio::main]
async fn main() {
    let stream = WSFeed::connect_with_auth(
            WS_SANDBOX_URL,
            &["BTC-USD"],
            &[ChannelType::Level2],
            KEY,
            SECRET,
            PASSPHRASE,
        )
        .await
        .unwrap();

    stream
        .take(10)
        .try_for_each(|msg| async {
            match msg {
                Message::Level2(
                    Level2::Snapshot {
                        product_id,
                        bids,
                        asks,
                    }
                ) => {
                    println!("--- [{} Order Book] ---", product_id);
                    println!("Bids: {:?}", bids);
                    println!("Asks: {:?}", asks);
                },
                Message::Level2(
                    Level2::L2update { changes, time, .. }
                ) => {
                    for update in changes {
                        println!("{}: {:?} {} @ {}", time, update.side, update.size, update.price)
                    }
                },
                Message::Error { message } => println!("Error: {}", message),
                Message::InternalError(_) => panic!("internal_error"),
                other => println!("{:?}", other),
            };
            Ok(())
        })
        .await
        .expect("stream fail");
}
