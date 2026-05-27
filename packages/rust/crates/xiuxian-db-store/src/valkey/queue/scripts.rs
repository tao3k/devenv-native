pub(super) const CLAIM_QUEUE_LUA: &str = r"
local pending_key = KEYS[1]
local lease_deadlines_key = KEYS[2]
local payload_prefix = ARGV[1]
local lease_prefix = ARGV[2]
local now_ms = tonumber(ARGV[3])
local lease_id = ARGV[4]
local worker_id = ARGV[5]
local acquired_at_ms = ARGV[6]
local expires_at_ms = ARGV[7]
local ttl_ms = tonumber(ARGV[8])
local filter_count = tonumber(ARGV[9])

local expired = redis.call('ZRANGEBYSCORE', lease_deadlines_key, '-inf', now_ms)
for _, entry_id in ipairs(expired) do
  local lease_key = lease_prefix .. entry_id
  local payload_key = payload_prefix .. entry_id
  if redis.call('EXISTS', lease_key) == 0 then
    local priority_score = redis.call('HGET', payload_key, 'priority_score')
    if priority_score then
      redis.call('ZADD', pending_key, priority_score, entry_id)
    end
    redis.call('ZREM', lease_deadlines_key, entry_id)
  else
    local active_expires_at_ms = tonumber(redis.call('HGET', lease_key, 'expires_at_ms') or '0')
    if active_expires_at_ms <= now_ms then
      redis.call('DEL', lease_key)
      local priority_score = redis.call('HGET', payload_key, 'priority_score')
      if priority_score then
        redis.call('ZADD', pending_key, priority_score, entry_id)
      end
      redis.call('ZREM', lease_deadlines_key, entry_id)
    end
  end
end

local candidates = redis.call('ZRANGE', pending_key, 0, -1)
for _, entry_id in ipairs(candidates) do
  local payload_key = payload_prefix .. entry_id
  local payload = redis.call('HGET', payload_key, 'payload')
  if not payload then
    redis.call('ZREM', pending_key, entry_id)
  else
    local not_before_ms = tonumber(redis.call('HGET', payload_key, 'not_before_ms') or '0')
    local matches_filters = 1
    local arg_index = 10
    for _ = 1, filter_count do
      local field_name = ARGV[arg_index]
      local field_value = ARGV[arg_index + 1]
      if (redis.call('HGET', payload_key, field_name) or '') ~= field_value then
        matches_filters = 0
        break
      end
      arg_index = arg_index + 2
    end
    if not_before_ms <= now_ms and matches_filters == 1 then
      local lease_key = lease_prefix .. entry_id
      redis.call('ZREM', pending_key, entry_id)
      redis.call(
        'HSET',
        lease_key,
        'lease_id',
        lease_id,
        'worker_id',
        worker_id,
        'acquired_at_ms',
        acquired_at_ms,
        'expires_at_ms',
        expires_at_ms
      )
      redis.call('PEXPIRE', lease_key, ttl_ms)
      redis.call('ZADD', lease_deadlines_key, expires_at_ms, entry_id)
      return {entry_id, payload}
    end
  end
end
return false
";

pub(super) const RENEW_LEASE_LUA: &str = r"
local lease_key = KEYS[1]
local lease_deadlines_key = KEYS[2]
local lease_id = ARGV[1]
local worker_id = ARGV[2]
local expires_at_ms = ARGV[3]
local ttl_ms = tonumber(ARGV[4])
local entry_id = ARGV[5]

if redis.call('EXISTS', lease_key) == 0 then
  return 0
end
if redis.call('HGET', lease_key, 'lease_id') ~= lease_id or redis.call('HGET', lease_key, 'worker_id') ~= worker_id then
  return -1
end
redis.call('HSET', lease_key, 'expires_at_ms', expires_at_ms)
redis.call('PEXPIRE', lease_key, ttl_ms)
redis.call('ZADD', lease_deadlines_key, expires_at_ms, entry_id)
return 1
";

pub(super) const RELEASE_LEASE_LUA: &str = r"
local lease_key = KEYS[1]
local lease_deadlines_key = KEYS[2]
local lease_id = ARGV[1]
local worker_id = ARGV[2]
local entry_id = ARGV[3]

if redis.call('EXISTS', lease_key) == 0 then
  return 0
end
if redis.call('HGET', lease_key, 'lease_id') ~= lease_id or redis.call('HGET', lease_key, 'worker_id') ~= worker_id then
  return -1
end
redis.call('DEL', lease_key)
redis.call('ZREM', lease_deadlines_key, entry_id)
return 1
";

pub(super) const RECLAIM_EXPIRED_LEASE_LUA: &str = r"
local pending_key = KEYS[1]
local lease_key = KEYS[2]
local lease_deadlines_key = KEYS[3]
local payload_key = KEYS[4]
local lease_id = ARGV[1]
local worker_id = ARGV[2]
local entry_id = ARGV[3]
local now_ms = tonumber(ARGV[4])

if redis.call('EXISTS', lease_key) == 0 then
  return 0
end
if redis.call('HGET', lease_key, 'lease_id') ~= lease_id or redis.call('HGET', lease_key, 'worker_id') ~= worker_id then
  return -1
end
local active_expires_at_ms = tonumber(redis.call('HGET', lease_key, 'expires_at_ms') or '0')
if active_expires_at_ms > now_ms then
  return 0
end
local priority_score = redis.call('HGET', payload_key, 'priority_score')
if not priority_score then
  return 0
end
redis.call('DEL', lease_key)
redis.call('ZREM', lease_deadlines_key, entry_id)
redis.call('ZADD', pending_key, priority_score, entry_id)
return 1
";
