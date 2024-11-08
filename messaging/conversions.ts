import * as borsh from "borsh";

export const convertRustSchemaToTS = (rustSchema: any): borsh.Schema => {
  const tsSchema: any = {};

  for (const [key, value] of Object.entries(rustSchema.definitions)) {
    if (value[1].Struct) {
      const fields: { [key: string]: borsh.Schema } = {};
      for (const [fieldName, fieldType] of value[1].Struct.fields.NamedFields) {
        fields[fieldName] = convertRustTypeToTS(fieldType);
      }
      tsSchema[key] = { struct: fields };
    } else if (value[1].Sequence) {
      if (value[1].Sequence.elements === "u8") {
        tsSchema[key] = "string";
      } else {
        tsSchema[key] = { array: { type: convertRustTypeToTS(value[1].Sequence.elements) } };
      }
    } else if (value[1].Primitive) {
      tsSchema[key] = convertRustPrimitiveToTS(value[1].Primitive[0]);
    } else {
      throw new Error(`Unsupported Rust schema type: ${JSON.stringify(value)}`);
    }
  }

  return tsSchema;
};

export const convertRustTypeToTS = (rustType: any): borsh.Schema => {
  if (typeof rustType === "string") {
    return convertRustPrimitiveToTS(rustType);
  } else if (Array.isArray(rustType) && rustType.length === 2 && typeof rustType[0] === "string" && typeof rustType[1] === "string") {
    return { struct: { [rustType[0]]: convertRustTypeToTS(rustType[1]) } };
  } else {
    throw new Error(`Unsupported Rust type: ${JSON.stringify(rustType)}`);
  }
};

export const convertRustPrimitiveToTS = (rustPrimitive: string | number): borsh.Schema => {
    if (typeof rustPrimitive === "number") {
      switch (rustPrimitive) {
        case 1:
          return "u8";
        case 2:
          return "u64";
        // Add additional mappings as needed for other types
        default:
          throw new Error(`Unsupported Rust primitive numeric code: ${rustPrimitive}`);
      }
    } else if (typeof rustPrimitive === "string") {
      switch (rustPrimitive) {
        case "u8":
          return "u8";
        case "u64":
          return "u64";
        case "String":
          return "string";
        default:
          throw new Error(`Unsupported Rust primitive type: ${rustPrimitive}`);
      }
    } else {
      throw new Error(`Unknown Rust primitive format: ${JSON.stringify(rustPrimitive)}`);
    }
  };