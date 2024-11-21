import * as dotenv from "dotenv";
import * as borsh from "borsh";
import fs from "fs";
import { convertRustSchemaToTS } from "./conversions";
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

class Person {
    public first_name: string;
    public last_name: string;

    constructor() {
        this.first_name = "jon";
        this.last_name = "doe";
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

        // ============== class with reading schema ==============
        const value3 = new Person();
        const schema3 = { struct: { first_name: "string", last_name: "string" }};
        const encoded3 = borsh.serialize(schema3, value3);
        console.log("Encoded3", encoded3);
        const schemaBuffer = fs.readFileSync("./person_schema0.dat");
        console.log("JSON Schema (Rust)", schemaBuffer);

        // get the ts schema
        // const tsSchema = convertRustSchemaToTS(JSON.parse(schemaBuffer.toString()));
        // console.log("TypeScript schema:", tsSchema);

        // const decoded3 = borsh.deserialize(tsSchema, encoded3);
        // console.log("Decoded3", decoded3);

        // ==========================================

        process.exit(0);
    } catch (ex) {
        console.error(`Error = ${ex}`);
        process.exit(-1);
    }
})();


