let chartInstances = {};
let devicesData = [];
let activeTabIndex = 0;
const gradients = {
    opsPerSec: ['#00144B', '#00BFFF'],
    latencyP99Ms: ['#00BFFF'],
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
            if (chartInstances.latencyP99MsChart) {
                chartInstances.latencyP99MsChart.resize();
                chartInstances.latencyP99MsChart.setOption(createLineChartOption(metricsData.latencyP99Ms, gradients.latencyP99Ms), true);
            }
        }
    });

    chartInstances = initCharts();
    await updateCharts(chartInstances);

    setInterval(() => updateCharts(chartInstances), 15000);
};

window.addEventListener('resize', function () {
    if (chartInstances.opsPerSecChart) chartInstances.opsPerSecChart.resize();
    if (chartInstances.latencyP99MsChart) chartInstances.latencyP99MsChart.resize();
    if (chartInstances.worldGraphChart) chartInstances.worldGraphChart.resize();
});


let metricsData = {
    readsPerSec: [], writesPerSec: [], latencyMeanMs: [], latencyP99Ms: []
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
                metricsData.latencyP99Ms.push([timestamp, item.latency_p99_ms]);

                totalReads += item.total_reads;
                totalWrites += item.total_writes;
                totalOps += (item.total_reads + item.total_writes);

                if (metricsData.readsPerSec.length > 300) {
                    metricsData.readsPerSec.shift();
                    metricsData.writesPerSec.shift();
                    metricsData.latencyP99Ms.shift();
                }

                ops_per_second = item.reads_per_second + item.writes_per_second
                document.getElementById('opsPerSec').innerText = ops_per_second.toLocaleString('en', {maximumFractionDigits: 0}) + " ops/sec";
                document.getElementById('readsPerSec').innerText = item.reads_per_second.toLocaleString('en', {maximumFractionDigits: 0}) + " reads/sec";
                document.getElementById('writesPerSec').innerText = item.writes_per_second.toLocaleString('en', {maximumFractionDigits: 0}) + " writes/sec";
                document.getElementById('latencyP99Ms').innerText = item.latency_p99_ms.toLocaleString('en', {maximumFractionDigits: 0}) + " ms";

                document.getElementById('totalOps').innerText = totalReads.toLocaleString('en', {maximumFractionDigits: 0}) + " total ops";
                document.getElementById('totalReads').innerText = totalReads.toLocaleString('en', {maximumFractionDigits: 0}) + " total reads";
                document.getElementById('totalWrites').innerText = totalWrites.toLocaleString('en', {maximumFractionDigits: 0}) + " total writes";
            }
        });

        devicesData = await fetchDevicesData();
    } catch (error) {
        console.error('Error fetching data:', error);
    }
}

function initCharts() {
    const opsPerSecChart = echarts.init(document.getElementById('opsPerSecChart'));
    const latencyP99MsChart = echarts.init(document.getElementById('latencyP99MsChart'));
    const worldGraphChart = echarts.init(document.getElementById('worldGraphChart'));

    return {
        opsPerSecChart, latencyP99MsChart, worldGraphChart
    };
}

async function updateCharts(chartInstances) {
    await fetchAndPrepareData();

    if (activeTabIndex === 0) {
        chartInstances.worldGraphChart.setOption(createWorldOption(), true);
        chartInstances.worldGraphChart.resize();
    }

    if (activeTabIndex === 1) {
        chartInstances.opsPerSecChart.setOption(createBarChartOption(metricsData.readsPerSec, metricsData.writesPerSec, gradients.readsPerSec), true);
        chartInstances.latencyP99MsChart.setOption(createLineChartOption(metricsData.latencyP99Ms, gradients.latencyP99Ms), true);
    }
}

function createWorldOption() {
    const graphData = devicesData.map((device, index, array) => {
        const nextDevice = array[index + 1] || array[0];
        return [[device.lat, device.lng], [nextDevice.lat, nextDevice.lng]];
    });

    const opt =  {
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
                    trailWidth: 2,
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

    return opt;
}

function createBarChartOption(readsData, writesData, gradientColors) {
    // Combine reads and writes data
    const combinedData = combineDataForStackedBar(readsData, writesData);

    return {
        tooltip: { trigger: 'axis' },
        xAxis: {
            type: 'category',
            data: combinedData.timestamps
        },
        yAxis: { type: 'value' },
        series: [
            {
                name: 'Reads',
                data: combinedData.reads,
                type: 'bar',
                stack: 'total',
                itemStyle: {
                    color: gradientColors[0]
                }
            },
            {
                name: 'Writes',
                data: combinedData.writes,
                type: 'bar',
                stack: 'total',
                itemStyle: {
                    color: gradientColors[1]
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


function createLineChartOption(data, gradientColors) {
    return {
        tooltip: { trigger: 'axis' },
        xAxis: {
            type: 'time',
            splitLine: { show: false }
        },
        yAxis: { type: 'value' },
        series: [{
            data: data,
            type: 'line',
            step: 'start',
            symbol: 'none',
            lineStyle: {
                opacity: 1,
                color: gradientColors[0]
            },
            itemStyle: {
                color: gradientColors[0]
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
