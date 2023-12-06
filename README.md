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

The demo application will now be running on port 8000. You can access the application by visiting http://localhost:8000/index.html.

![](demo.gif)

Note: the demo will simulate high load with simulated reads and writes, so you can expect to see the demo app 
consuming a lot of CPU. You can stop the demo at any time with:

    docker stop demo

Once you're happy with the demo, you can stop all the containers with:

    docker stop node1 demo
