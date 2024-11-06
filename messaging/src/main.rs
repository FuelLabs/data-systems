use anyhow::{Context, Result};
use borsh::{
    from_slice, to_vec, to_writer, try_from_slice_with_schema, try_to_vec_with_schema,
    BorshDeserialize, BorshSchema, BorshSerialize,
};
#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}

mod test {
    use super::*;

    #[test]
    fn test_no_schema() {
        #[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
        struct MyExampleStruct {
            name: String,
            age: u8,
        }

        let my_example_struct = MyExampleStruct { name: "Albert".to_string(), age: 9 };

        fn serialize_my_example_struct(my_example_struct: MyExampleStruct) -> Vec<u8> {
            let ser_my_example_struct: Vec<u8> = to_vec(&my_example_struct).unwrap();
            ser_my_example_struct
        }

        fn deserialize_my_example_struct(data: &[u8]) -> MyExampleStruct {
            let my_data = MyExampleStruct::try_from_slice(data).unwrap();
            my_data
        }

        // serialize
        let ser_my_example_struct: Vec<u8> = serialize_my_example_struct(my_example_struct.clone());
        println!("{:?}", ser_my_example_struct);
        // deserialize
        let deser_ser_my_example_struct = deserialize_my_example_struct(&ser_my_example_struct);
        println!("{:?}", deser_ser_my_example_struct);
        // assert
        assert_eq!(deser_ser_my_example_struct, my_example_struct);
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
        let ser_my_example_struct = try_to_vec_with_schema(&my_example_struct).unwrap();
        println!("{:?}", ser_my_example_struct);
        // Deserialize this instance from a slice of bytes, but assume that at the beginning we have
        // bytes describing the schema of the type
        let deser_ser_my_example_struct =
            try_from_slice_with_schema::<MyExampleStruct>(&ser_my_example_struct).unwrap();
        println!("{:?}", deser_ser_my_example_struct);
        // assert
        assert_eq!(deser_ser_my_example_struct, my_example_struct);
    }
}
