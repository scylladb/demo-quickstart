pub const DDL: &str = "
    CREATE KEYSPACE IF NOT EXISTS demo WITH REPLICATION = { 'class' : 'NetworkTopologyStrategy', 'replication_factor' : 3 };
    USE demo;
    DROP TABLE IF EXISTS metrics;
    CREATE TABLE IF NOT EXISTS metrics
    (
        node_id               text,
        timestamp             timestamp,
        reads_total           bigint,
        writes_total          bigint,
        latency_read_max      bigint,
        latency_write_max     bigint,
        PRIMARY KEY (node_id, timestamp)
    );
    USE demo;
    DROP TABLE IF EXISTS devices;
    CREATE TABLE IF NOT EXISTS devices (
        device_id   uuid,
        geo_hash    text,
        lat         double,
        lng         double,
        ipv4        text,
        PRIMARY KEY ((geo_hash), device_id)
    );
    USE demo;
    DROP TABLE IF EXISTS unique_lat_lng;
    CREATE TABLE IF NOT EXISTS unique_lat_lng (
        lat double,
        lng double,
        PRIMARY KEY (lat, lng)
    );
";
