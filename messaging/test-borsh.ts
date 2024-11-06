import * as dotenv from 'dotenv';
import {serialize, deserialize, Schema} from "borsh";

class Test {
    public x: number;
    public y: number;
    public z: string;
    public q: number[];

    constructor(properties: object) {
        Object.keys(properties).map((key) => {
            this[key] = properties[key];
        });
    }
}

dotenv.config();

(async () => {

    const header = "=".repeat(process.stdout.columns - 1);
    console.log(header);
    console.log(`${("Borsh Example")}`);
    console.log(header);

    try {
        const value = new Test({x: 255, y: 20, z: '123', q: [1, 2, 3]});
        const borshSchema = {
            kind: 'struct',
            fields: [
                ['x', 'u8'],
                ['y', 'u64'],
                ['z', 'string'],
                ['q', [3]]
            ]
        };
        const schema = new Map([[Test, borshSchema]]) as Schema;
        const buf: Uint8Array = serialize(schema, value);
        console.log(`Borsh serialized buffer ${buf}`);
        const newValue: Test = deserialize<Test>(schema, Test, Buffer.from(buf));
        console.log(`Borsh deserialized buffer (${newValue.x}, ${newValue.y}, ${newValue.z}, ${newValue.q})`);

        process.exit(0);
    } catch (ex) {
        console.error(`Error = ${ex}`);
        process.exit(-1);
    }
})();
