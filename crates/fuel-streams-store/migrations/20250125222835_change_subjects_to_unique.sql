-- Drop existing non-unique indexes
-- DROP INDEX IF EXISTS idx_blocks_subject;
-- DROP INDEX IF EXISTS idx_transactions_subject;
-- DROP INDEX IF EXISTS idx_inputs_subject;
-- DROP INDEX IF EXISTS idx_outputs_subject;
-- DROP INDEX IF EXISTS idx_utxos_subject;
-- DROP INDEX IF EXISTS idx_receipts_subject;

-- Add unique constraints
ALTER TABLE blocks ADD CONSTRAINT blocks_subject_unique UNIQUE (subject);
ALTER TABLE transactions ADD CONSTRAINT transactions_subject_unique UNIQUE (subject);
ALTER TABLE inputs ADD CONSTRAINT inputs_subject_unique UNIQUE (subject);
ALTER TABLE outputs ADD CONSTRAINT outputs_subject_unique UNIQUE (subject);
ALTER TABLE utxos ADD CONSTRAINT utxos_subject_unique UNIQUE (subject);
ALTER TABLE receipts ADD CONSTRAINT receipts_subject_unique UNIQUE (subject);
