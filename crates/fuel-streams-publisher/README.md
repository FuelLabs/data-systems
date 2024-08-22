## Fuel Streams Publisher

This binary subscribes to events emitted from a Fuel client or node to publish streams that can consumed via the `fuel-streams` SDK.

### Development

-   Generate the `KEYPAIR` environment variable using:

    -   ```
        fuel-core-keygen new --key-type peering -p
        ```

-   Generate an `INFURA_API_KEY` from https://app.infura.io/
-   Copy `.env.sample` and update the `KEYPAIR` AND `infura-api-key` generated above
-   From the monorepo's root, run this binary with `./scripts/start-publisher.sh`
-   Or using `make` and `docker`, using `make start/publisher`
