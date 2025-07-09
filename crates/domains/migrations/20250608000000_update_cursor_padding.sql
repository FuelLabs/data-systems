-- Splits our input on the delimiter
-- Leave the first element as it is
-- Pad all subsequential words to the given pad length
CREATE OR REPLACE FUNCTION split_and_pad(
    input_text TEXT,
    delimiter TEXT,
    pad_length INTEGER,
    pad_from INTEGER DEFAULT 0,
    pad_char TEXT DEFAULT '0'
)
RETURNS TEXT AS $$
BEGIN
    RETURN array_to_string(
        ARRAY(
            SELECT
                CASE
                    WHEN row_number() OVER () = pad_from THEN elem
                    ELSE lpad(elem, pad_length, pad_char)
                END
            FROM unnest(string_to_array(input_text, delimiter)) WITH ORDINALITY AS t(elem, ord)
            ORDER BY ord
        ),
        delimiter
    );
END;
$$ LANGUAGE plpgsql;

-- Update all tables with `cursor` and pad each part of the word
DO $$
DECLARE
    -- Skip over the first element (block_height) in our case
    -- We don't pad this element
    PAD_FROM CONSTANT INTEGER := 1;
    -- The padding length for all subsequential words after
    PAD_LENGTH CONSTANT INTEGER := 6;
    DELIMINATOR CONSTANT TEXT := '-';
BEGIN
    -- Update `inputs` table
    update inputs
    set cursor = split_and_pad(cursor, DELIMINATOR, PAD_LENGTH, PAD_FROM);

    -- Update `messages` table
    update messages
    set cursor = split_and_pad(cursor, DELIMINATOR, PAD_LENGTH, PAD_FROM);

    -- Update `transactions` table
    update transactions
    set cursor = split_and_pad(cursor, DELIMINATOR, PAD_LENGTH, PAD_FROM);

    -- Update `receipts` table
    update receipts
    set cursor = split_and_pad(cursor, DELIMINATOR, PAD_LENGTH, PAD_FROM);

    -- Update `utxos` table
    update utxos
    set cursor = split_and_pad(cursor, DELIMINATOR, PAD_LENGTH, PAD_FROM);

    -- Update `predicate_transactions` table
    update predicate_transactions
    set cursor = split_and_pad(cursor, DELIMINATOR, PAD_LENGTH, PAD_FROM);

    -- Update `outputs` table
    update outputs
    set cursor = split_and_pad(cursor, DELIMINATOR, PAD_LENGTH, PAD_FROM);
END $$;
