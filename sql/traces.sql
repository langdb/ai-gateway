CREATE TABLE IF NOT EXISTS langdb.traces
(
    trace_id        UUID,
    span_id         UInt64,
    parent_span_id  UInt64,
    operation_name  LowCardinality(String),
    kind            String,
    start_time_us   UInt64,
    finish_time_us  UInt64,
    finish_date     Date,
    attribute       Map(String, String),
    tenant_id       Nullable(String),
    project_id      String,
    thread_id       String,
    tags            Map(String, String),
    parent_trace_id Nullable(UUID),
    run_id          Nullable(UUID)
)
ENGINE = MergeTree
ORDER BY (finish_date, finish_time_us, trace_id)
SETTINGS index_granularity = 8192;

-- Add bloom filter index for thread_id
ALTER TABLE langdb.traces ADD INDEX idx_thread_id thread_id TYPE bloom_filter GRANULARITY 4;

-- Add composite index for tenant_id, project_id, and operation_name
ALTER TABLE langdb.traces ADD INDEX idx_tenant_project_op (tenant_id, project_id, operation_name) TYPE bloom_filter GRANULARITY 4;
