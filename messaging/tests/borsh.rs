use borsh::{schema::BorshSchemaContainer, BorshDeserialize, BorshSchema, BorshSerialize};
use borsh_schema_writer::schema_writer::write_schema;
use borsh_serde_adapter::{
    deserialize_adapter::deserialize_from_schema, serialize_adapter::serialize_serde_json_to_borsh,
};
use std::{fs::File, io::BufReader};

#[cfg(test)]
mod test {
    use std::io::Write;

    use borsh_serde_adapter::borsh_schema_util::write_schema_as_json;
    use messaging::generate_test_block;

    use super::*;

    #[test]
    fn test_no_schema() {
        #[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
        struct MyExampleStruct {
            name: String,
            age: u8,
        }

        let my_example_struct = MyExampleStruct { name: "Albert".to_string(), age: 9 };

        // serialize
        let serialized: Vec<u8> = borsh::to_vec(&my_example_struct).unwrap();
        // deserialize
        // let my_data = MyExampleStruct::try_from_slice(data).unwrap();
        // or
        let deserialized = borsh::from_slice::<MyExampleStruct>(&serialized).unwrap();
        // assert
        assert_eq!(deserialized, my_example_struct);
    }

    #[test]
    fn test_generate_schema() {
        #[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
        struct MyExampleStruct {
            name: String,
            age: u8,
        }

        MyExampleStruct::add_definitions_recursively(&mut Default::default());
        let container: BorshSchemaContainer = BorshSchemaContainer::for_type::<MyExampleStruct>();
        let schema = borsh::to_vec(&container).unwrap();
        println!("Generated Schema: {:?}", schema);
        let mut file = File::create("schema.dat").expect("Failed to create borsh schema file");
        file.write_all(&schema).expect("Failed to write file");
    }

    #[test]
    fn test_with_schema() {
        #[derive(BorshSerialize, BorshDeserialize, BorshSchema, PartialEq, Debug, Clone)]
        struct MyExampleStruct {
            name: String,
            age: u8,
        }

        let my_example_struct = MyExampleStruct { name: "Albert".to_string(), age: 9 };
        // Serialize object into a vector of bytes and prefix with the schema serialized as vector
        // of bytes in Borsh format.
        let serialized = borsh::try_to_vec_with_schema(&my_example_struct).unwrap();
        // Deserialize this instance from a slice of bytes, but assume that at the beginning we have
        // bytes describing the schema of the type
        let deserialized =
            borsh::try_from_slice_with_schema::<MyExampleStruct>(&serialized).unwrap();
        // assert
        assert_eq!(deserialized, my_example_struct);
    }

    #[test]
    fn test_schema_to_json_exported() {
        #[derive(Debug, Default, BorshSerialize, BorshDeserialize, BorshSchema)]
        pub struct Person {
            first_name: String,
            last_name: String,
        }

        write_schema_as_json(Person::default(), "./person_schema0.dat".to_string()).unwrap();
        let file = File::open("./person_schema0.dat").unwrap();
        let reader = BufReader::new(file);
        let result: serde_json::Value =
            serde_json::from_reader(reader).expect("Deserialization failed");
        println!("Schema as json: {}", result.to_string());
    }

    #[test]
    fn serialize_from_borsh_schema() {
        #[derive(Debug, Default, BorshSerialize, BorshDeserialize, BorshSchema)]
        pub struct Person {
            first_name: String,
            last_name: String,
        }

        write_schema(Person::default(), "./person_schema1.dat".to_string()).unwrap();

        let file = File::open("./person_schema1.dat").unwrap();
        let mut reader = BufReader::new(file);
        let person_schema = BorshSchemaContainer::deserialize_reader(&mut reader)
            .expect("Deserializing BorshSchemaContainer failed.");

        let person_value = serde_json::json!({"first_name": "John", "last_name": "Doe"});

        let mut person_writer = Vec::new();
        let _ = serialize_serde_json_to_borsh(&mut person_writer, &person_value, &person_schema)
            .expect("Serialization failed");

        let deserialized = borsh::from_slice::<Person>(&person_writer).unwrap();
        println!("Deserialized borsh: {:?}", deserialized);

        let result = deserialize_from_schema(&mut person_writer.as_slice(), &person_schema)
            .expect("Deserialization failed");
        println!("Deserialized json 1: {:?}", serde_json::to_string_pretty(&result));
    }

    #[test]
    fn deserialize_to_json_from_borsh_schema() {
        #[derive(Debug, Default, BorshSerialize, BorshDeserialize, BorshSchema)]
        pub struct Person {
            first_name: String,
            last_name: String,
        }
        let person = Person { first_name: "John".to_string(), last_name: "Doe".to_string() };
        write_schema(Person::default(), "./person_schema2.dat".to_string()).unwrap();

        let person_ser = borsh::to_vec(&person).unwrap();

        let file = File::open("./person_schema2.dat").unwrap();
        let mut reader = BufReader::new(file);
        let person_schema = BorshSchemaContainer::deserialize_reader(&mut reader)
            .expect("Deserializing BorshSchemaContainer failed.");
        let result = deserialize_from_schema(&mut person_ser.as_slice(), &person_schema)
            .expect("Deserializing from schema failed.");
        println!("Deserialized json 2: {:?}", serde_json::to_string_pretty(&result));
    }

    #[test]
    fn test_fuel_block() {
        let fuel_block = generate_test_block();
        let fuel_block_json = serde_json::to_string_pretty(&fuel_block).unwrap();
        let mut file = File::create("fuel_block.json").expect("Failed to write fuel block to json");
        file.write_all(fuel_block_json.as_bytes()).expect("Failed to write fuel block");
    }
}
