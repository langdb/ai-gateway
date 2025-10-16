### Threads API

**Threads** are logical groupings of related spans in the tracing system. A thread is defined as a collection of root spans (spans with a `thread_id` but no `parent_span_id`) that share the same `thread_id`.

#### Endpoints

- **GET /threads**: List threads for a project
  - Query params: `limit`, `offset` (optional)
  - Returns: List of threads (grouped root spans) ordered by start time (most recent first)
  - Response includes pagination metadata (offset, limit, total)

- **POST /threads**: List threads with body parameters (same as GET)
  - Body: `{ "page_options": { "limit": 50, "offset": 0 } }` (optional)
  - Returns: Same as GET endpoint

- **PUT /threads/{id}**: Update thread metadata
  - Path param: `id` (thread_id UUID)
  - Body: `{ "title": "New Title" }` (optional)
  - Returns: Updated thread object

#### Notes
- Auth and project context required (via X-Project-Id header or default project)
- All endpoints require valid project context from middleware
- Threads are automatically derived from traces table, not a separate threads table
- Pagination is applied at the database level for accurate results

### Schema

Thread response type: `ai-gateway/gateway/src/handlers/threads.rs::ThreadSpan`

#### ThreadSpan Fields:
- `thread_id` (string) - The unique thread identifier
- `start_time_us` (integer) - Earliest start time across all root spans in microseconds (MIN aggregation)
- `finish_time_us` (integer) - Latest finish time across all root spans in microseconds (MAX aggregation)
- `run_ids` (array<string>) - Array of unique run IDs associated with the thread's root spans

#### Response Format:
```json
{
  "data": [
    {
      "thread_id": "thread-123",
      "start_time_us": 1704067200000000,
      "finish_time_us": 1704067300000000,
      "run_ids": ["run-abc", "run-def"]
    }
  ],
  "pagination": {
    "offset": 0,
    "limit": 50,
    "total": 145
  }
}
```

#### Example Thread:
```json
{
  "thread_id": "f8b9c1d2-3456-7890-abcd-ef0123456789",
  "start_time_us": 1704067200000000,
  "finish_time_us": 1704067300000000,
  "run_ids": ["run-001", "run-002"]
}
```

### How Threads Work

Threads are derived from the `traces` table using the following SQL logic:

```sql
SELECT
    thread_id,
    MIN(start_time_us) as start_time_us,
    MAX(finish_time_us) as finish_time_us,
    GROUP_CONCAT(DISTINCT run_id) as run_ids
FROM traces
WHERE project_id = ?
    AND thread_id IS NOT NULL
    AND parent_span_id IS NULL
GROUP BY thread_id
ORDER BY start_time_us DESC
```

This means:
- A thread consists of all root spans (no parent) that share the same `thread_id`
- Multiple root spans can belong to the same thread
- Start/finish times represent the full duration across all root spans
- Run IDs are collected from all root spans in the thread
