-- 创建原始事件表
CREATE TABLE analytics_events (
    -- EventContext
    client_id String,
    session_id String,
    duration UInt32,
    app_version String,
    system_os String,
    system_arch String,
    system_language String,
    system_timezone String,
    user_id Nullable(String),
    ip_address Nullable(String),
    user_agent Nullable(String),
    referer Nullable(String),
    geo_country Nullable(String),
    geo_region Nullable(String),
    geo_city Nullable(String),
    client_ts DateTime64(3),
    server_ts DateTime64(3),
    -- event_type oneof
    event_type Enum8(
        'unspecified' = 0,
        'app_start' = 1,
        'app_exit' = 2,
        'user_login' = 3,
        'user_logout' = 4,
        'user_register' = 5,
        'message_sent' = 6,
        'chat_created' = 7,
        'chat_joined' = 8,
        'chat_left' = 9,
        'navigation' = 10
    ),
    -- AppExitEvent
    app_exit_code Nullable(
        Enum8(
            'EXIT_CODE_UNKNOWN' = 0,
            'EXIT_CODE_SUCCESS' = 1,
            'EXIT_CODE_FAILURE' = 2
        )
    ),
    -- UserLoginEvent/UserLogoutEvent/UserRegisterEvent
    email Nullable(String),
    -- UserRegisterEvent
    register_workspace_id Nullable(String),
    -- ChatCreatedEvent
    chat_created_workspace_id Nullable(String),
    -- MessageSentEvent
    message_sent_chat_id Nullable(String),
    message_sent_type Nullable(String),
    message_sent_size Nullable(Int32),
    message_sent_total_files Nullable(Int32),
    -- ChatJoinedEvent/ChatLeftEvent
    chat_joined_id Nullable(String),
    chat_left_id Nullable(String),
    -- NavigationEvent
    navigation_from Nullable(String),
    navigation_to Nullable(String)
) ENGINE = MergeTree
ORDER BY (event_type, session_id, client_id, server_ts);


-- 创建聚合后的sessions表
CREATE TABLE aggregated_sessions (
    date Date,
    client_id String,
    session_id String,
    session_start SimpleAggregateFunction(min, DateTime64(3)),
    session_end SimpleAggregateFunction(max, DateTime64(3)),
    total_events AggregateFunction(count, UInt64),
) ENGINE = AggregatingMergeTree
ORDER BY (date, client_id, session_id);
-- 创建物化视图以聚合数据并存储到目标表
CREATE MATERIALIZED VIEW aggregated_sessions_mv TO aggregated_sessions AS
SELECT toDate(server_ts) AS date,
    client_id,
    session_id,
    min(server_ts) AS session_start,
    max(server_ts) AS session_end,
    countState(1) AS total_events
FROM analytics_events
GROUP BY date,
    client_id,
    session_id;

-- 插入数据
insert into aggregated_sessions
SELECT toDate(server_ts) AS date,
    client_id,
    session_id,
    min(server_ts) AS session_start,
    max(server_ts) AS session_end,
    countState(1) AS total_events
FROM analytics_events
GROUP BY date,
    client_id,
    session_id;

-- 查询数据
select date,
    client_id,
    session_id,
    min(session_start) as session_start,
    max(session_end) as session_end,
    dateDiff('second', session_start, session_end) as duration,
    countMerge(total_events) as total_events
from aggregated_sessions
group by date,
    client_id,
    session_id
order by date desc,
    client_id,
    session_id

create table aggregated_daily_sessions(
    day Date,
    client_id String,
    total_sessions AggregateFunction(count, UInt64),
    total_duration AggregateFunction(sum, UInt64)
) ENGINE = AggregatingMergeTree
ORDER BY (day, client_id);

create materialized view aggregated_daily_sessions_mv to aggregated_daily_sessions as
select toDate(session_start) as day,
    client_id,
    countState(1) as total_sessions,
    sumState(toUInt64(dateDiff('second', session_start, session_end))) as total_duration
from aggregated_sessions
group by day,
    client_id;

insert into aggregated_daily_sessions
select toDate(session_start) as day,
    client_id,
    countState(1) as total_sessions,
    sumState(toUInt64(dateDiff('second', session_start, session_end))) as total_duration
from aggregated_sessions
group by day,
    client_id;

-- 查询数据
select day,
    client_id,
    countMerge(total_sessions) as total_sessions,
    sumMerge(total_duration) as total_duration
from aggregated_daily_sessions
group by day,
    client_id
order by day desc,
    client_id