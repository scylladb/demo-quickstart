# ScyllaDB Quickstart Demo

This is a quick start demonstration of ScyllaDB with Docker.

To run the demo for yourself. First start a single node cluster with the following command:

    docker run -d --rm --name node1 scylladb/scylla

Wait 60s or so for the node to start. Tip: you can view ScyllaDB logs with:

    docker logs -f node1

Next, run the demonstration application which will simulate artificial load from an Internet of Things app,
measuring data from millions of unique devices:

    docker run -d --rm --link node1:node1 \
        --publish 8000:8000 \
        --env DATABASE_URL=node1:9042 \
        --env METRICS_URL=node1:9180 \
        --name demo scylladb/demo-quickstart

The demo application will now be running on port 8000. You can access the application by
visiting http://localhost:8000/index.html.

![](demo.gif)

Note: the demo will simulate high load with simulated reads and writes, so you can expect to see the demo app
consuming a lot of CPU. You can stop the demo at any time with:

    docker stop demo

Once you're happy with the demo, you can stop all the containers with:

    docker stop node1 demo

# Options

## Datacenter

To adjust the preferred datacenter you can do so by setting the `DATACENTER` environment variable.

    docker run -d --rm --link node1:node1 \
        --publish 8000:8000 \
        --env DATABASE_URL=node1:9042 \
        --env METRICS_URL=node1:9180 \
        --env DATACENTER=dc2 \
        --name demo scylladb/demo-quickstart

Default is `datacenter1`.

## Consistency Level

To adjust consistency level you can do so by setting the `CONSISTENCY` environment variable.

    docker run -d --rm --link node1:node1 \
        --publish 8000:8000 \
        --env DATABASE_URL=node1:9042 \
        --env METRICS_URL=node1:9180 \
        --env CL=ONE \
        --name demo scylladb/demo-quickstart

Supported consistency levels are:

    "ONE" => Consistency::One,
    "TWO" => Consistency::Two,
    "THREE" => Consistency::Three,
    "QUORUM" => Consistency::Quorum,
    "ALL" => Consistency::All,
    "LOCAL_QUORUM" => Consistency::LocalQuorum,
    "EACH_QUORUM" => Consistency::EachQuorum,
    "SERIAL" => Consistency::Serial,
    "LOCAL_SERIAL" => Consistency::LocalSerial,
    "LOCAL_ONE" => Consistency::LocalOne,

Default is `LOCAL_QUORUM`.

## Load Profile

You can adjust the load profile and concurrency by setting args:

    docker run -d --rm --link node1:node1 \
    --publish 8000:8000   \
    --env DATABASE_URL=node1:9042 \
    --env METRICS_URL=node1:9180 \
    --name demo scylladb/demo-quickstart sh -c "/app/scylladb-quick-demo-rs 90 10 5"

The arguments are `% READS`, `% WRITES`, `SESSIONS`. Default is `80 20 30`. Ratio sum must be 100.