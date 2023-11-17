pub const DDL: &str = "
    CREATE KEYSPACE IF NOT EXISTS demo WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 };
    USE demo;
    DROP TABLE IF EXISTS metrics;
    CREATE TABLE IF NOT EXISTS metrics
    (
        node_id               text,
        timestamp             timestamp,
        queries_num           bigint,
        queries_iter_num      bigint,
        errors_num            bigint,
        errors_iter_num       bigint,
        latency_avg_ms        bigint,
        latency_percentile_ms bigint,
        PRIMARY KEY (node_id, timestamp)
    ) WITH CLUSTERING ORDER BY (timestamp ASC)
    AND COMPACTION = {'class': 'TimeWindowCompactionStrategy', 'base_time_seconds': 3600, 'max_sstable_age_days': 1};
    USE demo;
    DROP TABLE IF EXISTS devices;
    CREATE TABLE IF NOT EXISTS devices (
                                            device_id uuid,
                                            timestamp timestamp,
                                            sensor_data bigint,
                                            lat double,
                                            lng double,
                                            ipv4 text,
                                            PRIMARY KEY (device_id, timestamp)
    ) WITH CLUSTERING ORDER BY (timestamp ASC)
    AND COMPACTION = {'class': 'TimeWindowCompactionStrategy', 'base_time_seconds': 3600, 'max_sstable_age_days': 1};
";
