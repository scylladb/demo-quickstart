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

## Migrate

To skip DDL migration you can do so by setting the `MIGRATE` environment variable.

    docker run -d --rm --link node1:node1 \
        --publish 8000:8000 \
        --env DATABASE_URL=node1:9042 \
        --env METRICS_URL=node1:9180 \
        --env MIGRATE=false \
        --name demo scylladb/demo-quickstart

Default is `true`.

## Datacenter

To adjust the preferred datacenter you can do so by setting the `DATACENTER` environment variable.

    docker run -d --rm --link node1:node1 \
        --publish 8000:8000 \
        --env DATABASE_URL=node1:9042 \
        --env METRICS_URL=node1:9180 \
        --env DATACENTER=dc2 \
        --name demo scylladb/demo-quickstart

Default is `datacenter1`.

## Replication Factor

To adjust the replication factor you can do so by setting the `RF` environment variable.

    docker run -d --rm --link node1:node1 \
        --publish 8000:8000 \
        --env DATABASE_URL=node1:9042 \
        --env METRICS_URL=node1:9180 \
        --env RF=2 \
        --name demo scylladb/demo-quickstart

Default is `1` which is not recommended for production, but is fine for a single node demo.

## Consistency Level

To adjust consistency level you can do so by setting the `CL` environment variable.

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

## Multi Node Environment

You can also run this demo in a multi-node environment. 
Instead of creating a linked default network to node1 and publishing port 9042, 
we will instead create a bridge network and use the `--network` flag to connect the containers.

    docker network create --driver bridge scylla

Next start your multi node cluster:

    docker run -it --rm -d --name node1 --network scylla scylladb/scylla:6.1.1 --smp 1 --memory 1G
    docker run -it --rm -d --name node2 --network scylla scylladb/scylla:6.1.1 --smp 1 --memory 1G --seeds node1
    docker run -it --rm -d --name node3 --network scylla scylladb/scylla:6.1.1 --smp 1 --memory 1G --seeds node1

Now run the demo application:

    docker run -d --rm --network scylla \
        --publish 8000:8000 \
        --env DATABASE_URL=node1:9042,node2:9042,node3:9042 \
        --env METRICS_URL=node1:9180 \
        --name demo scylladb/demo-quickstart

This will start the demo application and connect to the multi-node cluster on the `scylla` network.
