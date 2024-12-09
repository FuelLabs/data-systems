# Running

1. First make sure you have your `.env` configured properly:

    ```sh
    just create-env
    ```

2. Make sure you have NATS server running within the workspace root:

    ```sh
    just start-nats
    ```

3. The, you can start local node and start publishing on NATS:
    ```sh
    just run-publisher
    ```
