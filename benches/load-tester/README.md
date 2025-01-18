# Running

To run the load-test suite:

    ```sh
    cargo run -- --network staging --ws-url "wss://stream-staging.fuel.network" --api-key "your_api_key" --max-subscriptions 10 --step-size 1
    ```

Adjustments are to be applied based on the max-subscriptions and step-size.
