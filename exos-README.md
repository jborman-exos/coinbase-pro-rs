# Getting Started

1. Start the container: `docker compose up`
2. Attach a terminal session to the container:
   1. If using VSCode's Dev Containers extension: `Ctrl+Shift+P` and search `Attach to Running Container`
3. Run the L2 Websocket example: `cargo run --example websocket_l2`:

```
    Finished dev [unoptimized + debuginfo] target(s) in 4m 32s
     Running `/cargo_target/debug/examples/websocket_l2`
Subscriptions { channels: [WithProduct { name: Level2, product_ids: ["BTC-USD"] }] }
--- [BTC-USD Order Book] ---
Bids: [Level2SnapshotRecord { price: 102.54, size: 0.02140494 }, Level2SnapshotRecord { price: 101.05, size: 0.02094209 }, ...]
Asks: [Level2SnapshotRecord { price: 102.78, size: 0.07433498 }, Level2SnapshotRecord { price: 102.79, size: 0.02140494 }, ...]

2023-10-26 19:02:41.336109 UTC: Buy 0 @ 102.54
2023-10-26 19:02:41.371622 UTC: Sell 0.02140494 @ 107.23
2023-10-26 19:02:41.674012 UTC: Buy 0 @ 101.05
2023-10-26 19:02:41.674012 UTC: Buy 0.02089199 @ 100.84
2023-10-26 19:02:41.674013 UTC: Buy 0 @ 100.84
2023-10-26 19:02:41.674013 UTC: Buy 0.02084189 @ 100.7
2023-10-26 19:02:41.674014 UTC: Buy 0 @ 100.7
2023-10-26 19:02:41.674014 UTC: Buy 0.02079179 @ 100.42
```