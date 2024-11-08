import * as dotenv from "dotenv";
import { PubKeyRequest, PubKeyResponse, Request } from "./protobuftypes";

dotenv.config();

function isValidRequest(request: Request): boolean {
    return (
      (request.pubKeyRequest !== undefined) !== (request.pubKeyResponse !== undefined)
    );
  }

  function handleRequest(request: Request) {
    if (request.pubKeyRequest) {
      console.log("Handling PubKeyRequest:", request.pubKeyRequest.id);
    } else if (request.pubKeyResponse) {
      console.log("Handling PubKeyResponse:", request.pubKeyResponse.id);
    } else {
      throw new Error("Invalid Request: Both fields are undefined");
    }
  }
(async () => {

    const header = "=".repeat(process.stdout.columns - 1);
    console.log(header);
    console.log(`${("Protobuf Example")}`);
    console.log(header);

    try {

        const pubKeyRequest: PubKeyRequest = {
            id: "unique-request-id",
            data: new Uint8Array([0x12, 0x34, 0x56]),
          };

          const requestWithPubKeyRequest: Request = {
            pubKeyRequest,
          };


          const pubKeyResponse: PubKeyResponse = {
            id: "unique-response-id",
            data: new Uint8Array([0x78, 0x9A, 0xBC]),
          };

          const requestWithPubKeyResponse: Request = {
            pubKeyResponse,
          };

        // Encoding a Request
        const encodedRequest = Request.encode(requestWithPubKeyRequest).finish();
        // Decoding a Request
        const decodedRequest = Request.decode(encodedRequest);
        handleRequest(decodedRequest);

        process.exit(0);
    } catch (ex) {
        console.error(`Error = ${ex}`);
        process.exit(-1);
    }
})();


