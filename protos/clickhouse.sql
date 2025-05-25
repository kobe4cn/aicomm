CREATE TABLE analytics_events (
    -- EventContext
    client_id String,
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
ORDER BY (server_ts, client_id);
