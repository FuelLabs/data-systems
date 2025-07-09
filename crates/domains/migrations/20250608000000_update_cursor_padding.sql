-- Splits our input on the delimiter and pad to length
CREATE OR REPLACE FUNCTION split_and_pad(
    input_text TEXT,
    delimiter TEXT,
    pad_length INTEGER,
    pad_char TEXT DEFAULT '0'
)
RETURNS TEXT AS $$
BEGIN
    RETURN array_to_string(
        ARRAY(
            SELECT lpad(unnest(string_to_array(input_text, delimiter)), pad_length, pad_char)
        ),
        delimiter
    );
END;
$$ LANGUAGE plpgsql;

-- Update all tables with `cursor` and pad each part of the word
DO $$
DECLARE
    -- Constants for the new cursor word length
    WORD_LENGTH CONSTANT INTEGER := 10;
    DELIMINATOR CONSTANT TEXT := '-';
BEGIN
    -- Update `inputs` table
    update inputs
    set cursor = split_and_pad(cursor, DELIMINATOR, WORD_LENGTH);

    -- Update `messages` table
    update messages
    set cursor = split_and_pad(cursor, DELIMINATOR, WORD_LENGTH);

    -- Update `transactions` table
    update transactions
    set cursor = split_and_pad(cursor, DELIMINATOR, WORD_LENGTH);

    -- Update `receipts` table
    update receipts
    set cursor = split_and_pad(cursor, DELIMINATOR, WORD_LENGTH);

    -- Update `utxos` table
    update utxos
    set cursor = split_and_pad(cursor, DELIMINATOR, WORD_LENGTH);

    -- Update `predicate_transactions` table
    update predicate_transactions
    set cursor = split_and_pad(cursor, DELIMINATOR, WORD_LENGTH);

    -- Update `outputs` table
    update outputs
    set cursor = split_and_pad(cursor, DELIMINATOR, WORD_LENGTH);
END $$;
