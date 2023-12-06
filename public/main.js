let chartInstances = {};
let devicesData = [];
let activeTabIndex = 0;
const gradients = {
    opsPerSec: ['#00144B', '#00BFFF'],
    latency: ['#00144B', '#00BFFF'],
};
window.onload = async () => {
    mdc.autoInit();

    mdc.tabBar.MDCTabBar.attachTo(document.querySelector('.mdc-tab-bar')).listen('MDCTabBar:activated', function (event) {
        document.querySelector('.panel.active').classList.remove('active');
        document.querySelector('#panel-container .panel:nth-child(' + (event.detail.index + 1) + ')').classList.add('active');
        activeTabIndex = event.detail.index;

        if (event.detail.index === 1) {
            if (chartInstances.opsPerSecChart) {
                chartInstances.opsPerSecChart.resize();
                chartInstances.opsPerSecChart.setOption(createBarChartOption(metricsData.readsPerSec, metricsData.writesPerSec, gradients.opsPerSec), true);
            }
            if (chartInstances.latencyChart) {
                chartInstances.latencyChart.resize();
                chartInstances.latencyChart.setOption(createLineChartOption(metricsData.latencyReadMax, metricsData.latencyWriteMax, gradients.latency), true);
            }
        }
    });

    chartInstances = initCharts();
    await updateCharts(chartInstances);

    setInterval(() => updateCharts(chartInstances), 5000);
};

window.addEventListener('resize', function () {
    if (chartInstances.opsPerSecChart) chartInstances.opsPerSecChart.resize();
    if (chartInstances.latencyChart) chartInstances.latencyChart.resize();
    if (chartInstances.worldGraphChart) chartInstances.worldGraphChart.resize();
});


let metricsData = {
    readsPerSec: [], writesPerSec: [], latencyReadMax: [], latencyWriteMax: []
};

let totalReads = 0;
let totalWrites = 0;
let totalOps = 0;

async function fetchAndPrepareData() {
    try {
        const response = await fetch('/metrics');
        const data = await response.json();

        let lastTimestamp = 0;
        if (metricsData.readsPerSec.length > 0) {
            lastTimestamp = metricsData.readsPerSec[metricsData.readsPerSec.length - 1][0];
        }

        data.forEach(item => {
            const timestamp = item.timestamp;

            if (timestamp > lastTimestamp) {
                metricsData.readsPerSec.push([timestamp, item.reads_per_second]);
                metricsData.writesPerSec.push([timestamp, item.writes_per_second]);
                metricsData.latencyReadMax.push([timestamp, item.latency_read_max]);
                metricsData.latencyWriteMax.push([timestamp, item.latency_write_max]);

                totalReads = item.reads_total;
                totalWrites = item.writes_total;
                totalOps = item.reads_total + item.writes_total;

                if (metricsData.readsPerSec.length > 300) {
                    metricsData.readsPerSec.shift();
                    metricsData.writesPerSec.shift();
                    metricsData.latencyReadMax.shift();
                    metricsData.latencyWriteMax.shift();
                }

                ops_per_second = item.ops_per_second;
                document.getElementById('opsPerSec').innerText = ops_per_second.toLocaleString('en', {maximumFractionDigits: 0}) + " ops/sec";
            }
        });

        devicesData = await fetchDevicesData();
    } catch (error) {
        console.error('Error fetching data:', error);
    }
}

function initCharts() {
    const opsPerSecChart = echarts.init(document.getElementById('opsPerSecChart'));
    const latencyChart = echarts.init(document.getElementById('latencyChart'));
    const worldGraphChart = echarts.init(document.getElementById('worldGraphChart'));

    return {
        opsPerSecChart, latencyChart, worldGraphChart
    };
}

async function updateCharts(chartInstances) {
    await fetchAndPrepareData();

    if (activeTabIndex === 0) {
        chartInstances.worldGraphChart.setOption(createWorldOption(), true);
        chartInstances.worldGraphChart.resize();
    }

    if (activeTabIndex === 1) {
        chartInstances.opsPerSecChart.setOption(createBarChartOption(metricsData.readsPerSec, metricsData.writesPerSec, gradients.opsPerSec), true);
        chartInstances.opsPerSecChart.resize();
        chartInstances.latencyChart.setOption(createLineChartOption(metricsData.latencyReadMax, metricsData.latencyWriteMax, gradients.latency), true);
        chartInstances.latencyChart.resize();
    }
}

function createWorldOption() {
    const graphData = devicesData.map((device, index, array) => {
        const nextDevice = array[index + 1] || array[0];
        return [[device.lat, device.lng], [nextDevice.lat, nextDevice.lng]];
    });

    return {
        geo3D: {
            map: 'world',
            silent: true,
            environment: '#5c677d',
            postEffect: {
                enable: false
            },
            groundPlane: {
                show: false,
            },

            viewControl: {
                distance: 80,
                    alpha: 90,
                    panMouseButton: 'left',
                    rotateMouseButton: 'right'
            },
            itemStyle: {
                color: '#001233'
            },
            regionHeight: 0.5
        },
        series: [
            {
                type: 'lines3D',
                coordinateSystem: 'geo3D',
                effect: {
                    show: true,
                    trailWidth: 1.5,
                    trailOpacity: 0.5,
                    trailLength: 0.2,
                    constantSpeed: 5
                },
                blendMode: 'lighter',
                lineStyle: {
                    width: 0.2,
                    opacity: 0.05
                },
                data: graphData
            }
        ]
    };
}

function createBarChartOption(readsData, writesData, gradients) {
    // Combine reads and writes data
    const combinedData = combineDataForStackedBar(readsData, writesData);

    return {
        tooltip: { trigger: 'axis' },
        xAxis: {
            type: 'category',
            data: combinedData.timestamps
        },
        yAxis: { type: 'value' },
        legend: {
            data: ['Reads/sec', 'Writes/sec']
        },
        series: [
            {
                name: 'Reads/sec',
                data: combinedData.reads,
                type: 'bar',
                stack: 'total',
                itemStyle: {
                    color: gradients[0]
                }
            },
            {
                name: 'Writes/sec',
                data: combinedData.writes,
                type: 'bar',
                stack: 'total',
                itemStyle: {
                    color: gradients[1]
                }
            }
        ]
    };
}

function combineDataForStackedBar(readsData, writesData) {
    let timestamps = [];
    let reads = [];
    let writes = [];

    // Assuming readsData and writesData are aligned and of the same length
    for (let i = 0; i < readsData.length; i++) {
        timestamps.push(formatTimestamp(readsData[i][0])); // Format timestamp as needed
        reads.push(readsData[i][1]);
        writes.push(writesData[i][1]);
    }

    return { timestamps, reads, writes };
}

// Add a function to format timestamps if necessary
function formatTimestamp(timestamp) {
    // Format the timestamp as required for the xAxis
    // Example: return new Date(timestamp).toLocaleString();
    return new Date(timestamp).toLocaleString();
}


function createLineChartOption(readsData, writesData, gradients) {
    return {
        tooltip: { trigger: 'axis' },
        xAxis: {
            type: 'time',
            splitLine: { show: false }
        },
        yAxis: { type: 'value' },
        legend: {
            data: ['P99 Read Latency (ms)', 'P99 Write Latency (ms)']
        },
        series: [{
            name: 'P99 Read Latency (ms)',
            data: readsData,
            type: 'line',
            step: 'start',
            symbol: 'none',
            lineStyle: {
                opacity: 1,
                color: gradients[0]
            },
            itemStyle: {
                color: gradients[0]
            }
        },
        {
            name: 'P99 Write Latency (ms)',
            data: writesData,
            type: 'line',
            step: 'start',
            symbol: 'none',
            lineStyle: {
                opacity: 1,
                color: gradients[1]
            },
            itemStyle: {
                color: gradients[1]
            }
        }]
    };
}


async function fetchDevicesData() {
    try {
        const response = await fetch('/devices');
        return await response.json();
    } catch (error) {
        console.error('Error fetching devices data:', error);
        return [];
    }
}
