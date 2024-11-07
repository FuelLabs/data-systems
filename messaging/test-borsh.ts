import * as dotenv from "dotenv";
import * as borsh from "borsh";

class Test {
    public x: number;
    public y: number;
    public z: string;
    public q: number[];

    constructor() {
        this.x = 11;
        this.y = 22;
        this.z = "xxx";
        this.q = [1, 2, 3];
    }
}

dotenv.config();

(async () => {

    const header = "=".repeat(process.stdout.columns - 1);
    console.log(header);
    console.log(`${("Borsh Example")}`);
    console.log(header);

    try {
        // ============== js object ==============
        const value1 = {x: 255, y: BigInt(20), z: "123", arr: [1, 2, 3]};
        const schema1 = { struct: { x: "u8", y: "u64", "z": "string", "arr": { array: { type: "u8" }}}};
        const encoded1 = borsh.serialize(schema1, value1);
        console.log("Encoded 1", encoded1);
        const decoded1 = borsh.deserialize(schema1, encoded1);
        console.log("Decoded 1", decoded1);


        // ============== class with constructor ==============
        const value2 = new Test();
        const schema2 = { struct: { x: "u8", y: "u64", "z": "string", "q": { array: { type: "u8" }}}};
        const encoded2 = borsh.serialize(schema2, value2);
        console.log("Encoded2", encoded2);
        const decoded2 = borsh.deserialize(schema2, encoded2);
        console.log("Decoded2", decoded2);

        process.exit(0);
    } catch (ex) {
        console.error(`Error = ${ex}`);
        process.exit(-1);
    }
})();
