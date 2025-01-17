# Running

To run the load-test suite:

    ```sh
    cargo run -- --nats-url "nats://localhost:4222" --db-url "postgresql://root@localhost:26257/defaultdb?sslmode=disable" --max-subscriptions 10 --step-size 1
    ```

Adjustments are to be applied based on the max-subscriptions and step-size.
