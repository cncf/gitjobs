import { prettifyNumber } from "/static/js/common/common.js";

/**
 * ECharts theme configuration for GitJobs charts.
 * Defines color schemes, styles, and visual properties.
 * @type {Object}
 */
export const gitjobsChartTheme = {
  color: ["#fd4d12", "#5470c6", "#91cc75", "#fac858", "#ee6666", "#73c0de", "#3ba272", "#fc8452", "#9a60b4"],
  backgroundColor: "rgba(0,0,0,0)",
  textStyle: {},
  title: {
    textStyle: {
      color: "#464646",
    },
    subtextStyle: {
      color: "#6e7079",
    },
  },
  line: {
    itemStyle: {
      borderWidth: 1,
    },
    lineStyle: {
      width: 2,
    },
    symbolSize: 4,
    symbol: "emptyCircle",
    smooth: false,
  },
  radar: {
    itemStyle: {
      borderWidth: 1,
    },
    lineStyle: {
      width: 2,
    },
    symbolSize: 4,
    symbol: "emptyCircle",
    smooth: false,
  },
  bar: {
    itemStyle: {
      barBorderWidth: 0,
      barBorderColor: "#ccc",
    },
  },
  pie: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
  },
  scatter: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
  },
  boxplot: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
  },
  parallel: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
  },
  sankey: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
  },
  funnel: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
  },
  gauge: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
  },
  candlestick: {
    itemStyle: {
      color: "#eb5454",
      color0: "#47b262",
      borderColor: "#eb5454",
      borderColor0: "#47b262",
      borderWidth: 1,
    },
  },
  graph: {
    itemStyle: {
      borderWidth: 0,
      borderColor: "#ccc",
    },
    lineStyle: {
      width: 1,
      color: "#aaa",
    },
    symbolSize: 4,
    symbol: "emptyCircle",
    smooth: false,
    color: [
      "#5470c6",
      "#91cc75",
      "#fac858",
      "#ee6666",
      "#73c0de",
      "#3ba272",
      "#fc8452",
      "#9a60b4",
      "#ea7ccc",
    ],
    label: {
      color: "#eeeeee",
    },
  },
  map: {
    itemStyle: {
      areaColor: "#eee",
      borderColor: "#444",
      borderWidth: 0.5,
    },
    label: {
      color: "#000",
    },
    emphasis: {
      itemStyle: {
        areaColor: "rgba(255,215,0,0.8)",
        borderColor: "#444",
        borderWidth: 1,
      },
      label: {
        color: "rgb(100,0,0)",
      },
    },
  },
  geo: {
    itemStyle: {
      areaColor: "#eee",
      borderColor: "#444",
      borderWidth: 0.5,
    },
    label: {
      color: "#000",
    },
    emphasis: {
      itemStyle: {
        areaColor: "rgba(255,215,0,0.8)",
        borderColor: "#444",
        borderWidth: 1,
      },
      label: {
        color: "rgb(100,0,0)",
      },
    },
  },
  categoryAxis: {
    axisLine: {
      show: true,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisTick: {
      show: true,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisLabel: {
      show: true,
      color: "#6E7079",
    },
    splitLine: {
      show: false,
      lineStyle: {
        color: ["#E0E6F1"],
      },
    },
    splitArea: {
      show: false,
      areaStyle: {
        color: ["rgba(250,250,250,0.2)", "rgba(210,219,238,0.2)"],
      },
    },
  },
  valueAxis: {
    axisLine: {
      show: false,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisTick: {
      show: false,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisLabel: {
      show: true,
      color: "#6E7079",
    },
    splitLine: {
      show: true,
      lineStyle: {
        color: ["#E0E6F1"],
      },
    },
    splitArea: {
      show: false,
      areaStyle: {
        color: ["rgba(250,250,250,0.2)", "rgba(210,219,238,0.2)"],
      },
    },
  },
  logAxis: {
    axisLine: {
      show: false,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisTick: {
      show: false,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisLabel: {
      show: true,
      color: "#6E7079",
    },
    splitLine: {
      show: true,
      lineStyle: {
        color: ["#E0E6F1"],
      },
    },
    splitArea: {
      show: false,
      areaStyle: {
        color: ["rgba(250,250,250,0.2)", "rgba(210,219,238,0.2)"],
      },
    },
  },
  timeAxis: {
    axisLine: {
      show: true,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisTick: {
      show: true,
      lineStyle: {
        color: "#6E7079",
      },
    },
    axisLabel: {
      show: true,
      color: "#6E7079",
    },
    splitLine: {
      show: false,
      lineStyle: {
        color: ["#E0E6F1"],
      },
    },
    splitArea: {
      show: false,
      areaStyle: {
        color: ["rgba(250,250,250,0.2)", "rgba(210,219,238,0.2)"],
      },
    },
  },
  toolbox: {
    iconStyle: {
      borderColor: "#999",
    },
    emphasis: {
      iconStyle: {
        borderColor: "#666",
      },
    },
  },
  legend: {
    textStyle: {
      color: "#333333",
    },
  },
  tooltip: {
    axisPointer: {
      lineStyle: {
        color: "#ccc",
        width: 1,
      },
      crossStyle: {
        color: "#ccc",
        width: 1,
      },
    },
  },
  timeline: {
    lineStyle: {
      color: "#dae1f5",
      width: 2,
    },
    itemStyle: {
      color: "#a4b1d7",
      borderWidth: 1,
    },
    controlStyle: {
      color: "#a4b1d7",
      borderColor: "#a4b1d7",
      borderWidth: 1,
    },
    checkpointStyle: {
      color: "#316bf3",
      borderColor: "#ffffff",
    },
    label: {
      color: "#a4b1d7",
    },
    emphasis: {
      itemStyle: {
        color: "#ffffff",
      },
      controlStyle: {
        color: "#a4b1d7",
        borderColor: "#a4b1d7",
        borderWidth: 1,
      },
      label: {
        color: "#a4b1d7",
      },
    },
  },
  visualMap: {
    color: ["#bf444c", "#d88273", "#f6efa6"],
  },
  dataZoom: {
    handleSize: "undefined%",
    textStyle: {},
  },
  markPoint: {
    label: {
      color: "#eeeeee",
    },
    emphasis: {
      label: {
        color: "#eeeeee",
      },
    },
  },
};

const MESSAGE_EMPTY_STATS = "No data available yet";

/**
 * Finds the smallest value in an array of numbers.
 * @param {Array<number>} numbers - Array of numbers to search
 * @returns {number} The smallest number in the array
 */
const getSmallestValue = (numbers) => {
  if (!numbers || numbers.length === 0) {
    throw new Error("Array is empty or undefined");
  }
  return Math.min(...numbers);
};

/**
 * Gets the minimum date value from the data array.
 * If the minimum date is less than the provided minimum value, returns that date.
 * Otherwise, returns the provided minimum value.
 * @param {Array} data - Array of data items where each item is an array with a timestamp as the first element
 * @param {number} min - Minimum date value to compare
 * @returns {number} The minimum date value from the data or the provided minimum value
 */
const getMinDateValue = (data, min) => {
  const dates = data.map((item) => item[0]);
  const minDate = getSmallestValue(dates);
  return minDate < min ? minDate : min;
};

/**
 * Finds the greatest value in an array of numbers.
 * @param {Array<number>} numbers - Array of numbers to search
 * @returns {number} The greatest number in the array
 */
const getGreatestValue = (numbers) => {
  if (!numbers || numbers.length === 0) {
    throw new Error("Array is empty or undefined");
  }
  return Math.max(...numbers);
};

/**
 * Gets the maximum date value from the data array.
 * If the maximum date is greater than the provided maximum value, returns that date.
 * Otherwise, returns the provided maximum value.
 * @param {Array} data - Array of data items where each item is an array with a timestamp as the first element
 * @param {number} max - Maximum date value to compare
 * @returns {number} The maximum date value from the data or the provided maximum value
 */
const getMaxDateValue = (data, max) => {
  const dates = data.map((item) => item[0]);
  const maxDate = getGreatestValue(dates);
  return maxDate > max ? maxDate : max;
};

/**
 * Renders a line chart showing job publication trends.
 * @param {Array} data - Time series data with timestamps and job counts
 * @private
 */
const renderLineChart = (data) => {
  const chartDom = document.getElementById("line-chart");
  if (!chartDom) return;

  const myChart = echarts.init(chartDom, "gitjobs", {
    renderer: "svg",
    useDirtyRect: false,
  });

  window.addEventListener("resize", function () {
    myChart.resize();
  });

  const option = {
    dataset: [
      {
        dimensions: ["timestamp", "jobs"],
        source: data,
      },
      {
        transform: {
          type: "sort",
          config: { dimension: "timestamp", order: "asc" },
        },
      },
    ],
    tooltip: {
      axisPointer: {
        type: "shadow",
      },
      formatter: (params) => {
        const chartdate = echarts.time.format(params.data[0], "{dd} {MMM} {yyyy}");
        return `<strong>${chartdate}</strong><br />Published jobs: ${params.data[1]}`;
      },
    },
    xAxis: {
      type: "category",
      axisLabel: {
        hideOverlap: true,
        formatter: (value) => {
          const date = echarts.time.format(parseInt(value), "{dd} {MMM}");
          return date;
        },
      },
      splitLine: {
        show: false,
      },
    },
    yAxis: {
      type: "value",
      axisLabel: {
        formatter: (value) => `${prettifyNumber(value)}`,
      },
    },
    series: {
      type: "line",
      name: "Published jobs",
      encode: { x: "timestamp", y: "jobs" },
      areaStyle: {},
      datasetIndex: 1,
      areaStyle: {
        color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
          {
            offset: 0,
            color: "rgb(253, 77, 18)",
          },
          {
            offset: 1,
            color: "rgb(255, 230, 212)",
          },
        ]),
      },
    },
    media: [
      {
        query: {
          maxWidth: 550,
        },
        option: {
          grid: {
            left: "60px",
          },
        },
      },
    ],
  };

  option && myChart.setOption(option);
};

/**
 * Returns common configuration options for bar charts.
 * @returns {Object} Base bar chart configuration
 */
export const getBarStatsOptions = () => {
  return {
    dataset: [],
    tooltip: {
      axisPointer: {
        type: "shadow",
      },
    },
    xAxis: {
      type: "time",
      scale: true,
      axisTick: {
        alignWithLabel: true,
      },
    },
    yAxis: {
      type: "value",
      axisLabel: {
        formatter: (value) => `${prettifyNumber(value)}`,
      },
    },
    series: {
      type: "bar",
      name: "Views",
      encode: { x: "timestamp", y: "jobs" },
      barMaxWidth: 35,
      label: {
        show: true,
        position: "top",
        formatter: (params) => {
          return prettifyNumber(params.value[1]);
        },
      },
      datasetIndex: 1,
    },
    media: [
      {
        query: {
          maxWidth: 800,
        },
        option: {
          series: {
            label: {
              show: false,
            },
          },
        },
      },
      {
        query: {
          maxWidth: 550,
        },
        option: {
          grid: {
            left: "60px",
          },
          series: {
            barMaxWidth: 10,
          },
        },
      },
    ],
  };
};

/**
 * Renders a bar chart showing daily job statistics.
 * @param {Array} data - Daily statistics data
 * @param {number} max - Maximum date value for x-axis
 * @param {number} min - Minimum date value for x-axis
 * @private
 */
const renderBarDailyChart = (data, max, min) => {
  const chartDom = document.getElementById("bar-daily");
  if (!chartDom) return;

  const myChart = echarts.init(chartDom, "gitjobs", {
    renderer: "svg",
    useDirtyRect: false,
  });

  window.addEventListener("resize", function () {
    myChart.resize();
  });

  const option = {
    ...getBarStatsOptions(),
    dataset: [
      {
        dimensions: ["timestamp", "jobs"],
        source: data,
      },
      {
        transform: {
          type: "sort",
          config: { dimension: "timestamp", order: "asc" },
        },
      },
    ],
    tooltip: {
      ...getBarStatsOptions().tooltip,
      formatter: (params) => {
        const chartdate = echarts.time.format(params.data[0], "{dd} {MMM}'{yy}");
        return `${chartdate}<br />Views: ${prettifyNumber(params.data[1])}`;
      },
    },
    xAxis: {
      ...getBarStatsOptions().xAxis,
      axisLabel: { interval: 0, formatter: "{dd} {MMM}", hideOverlap: true },
      splitLine: {
        show: false,
      },
      min: getMinDateValue(data, min),
      max: getMaxDateValue(data, max),
    },
  };
  option && myChart.setOption(option);
};

/**
 * Renders a bar chart showing monthly job statistics.
 * @param {Array} data - Monthly statistics data
 * @param {number} max - Maximum date value for x-axis
 * @param {number} min - Minimum date value for x-axis
 * @private
 */
const renderBarMonthlyChart = (data, max, min) => {
  const chartDom = document.getElementById("bar-monthly");
  if (!chartDom) return;

  const myChart = echarts.init(chartDom, "gitjobs", {
    renderer: "svg",
    useDirtyRect: false,
  });

  window.addEventListener("resize", function () {
    myChart.resize();
  });

  const option = {
    ...getBarStatsOptions(),
    dataset: [
      {
        dimensions: ["timestamp", "jobs"],
        source: data,
      },
      {
        transform: {
          type: "sort",
          config: { dimension: "timestamp", order: "asc" },
        },
      },
    ],
    tooltip: {
      ...getBarStatsOptions().tooltip,
      formatter: (params) => {
        const chartdate = echarts.time.format(params.data[0], "{MMM} {yyyy}");
        return `${chartdate}<br />Views: ${prettifyNumber(params.data[1])}`;
      },
    },
    xAxis: {
      ...getBarStatsOptions().xAxis,
      axisLabel: { interval: 0, formatter: "{MMM}'{yy}", hideOverlap: true },
      min: getMinDateValue(data, min),
      max: getMaxDateValue(data, max),
    },
  };
  option && myChart.setOption(option);
};

/**
 * Initializes and renders all statistics charts.
 * Reads data from DOM element and creates visualizations.
 */
export const renderStats = () => {
  const container = document.getElementById("stats");
  if (!container) return;

  const data = container.dataset.stats;
  if (!data) return;

  const stats = JSON.parse(data);

  if (!stats) return;

  // Register the GitJobs theme for ECharts
  echarts.registerTheme("gitjobs", gitjobsChartTheme);

  if (!stats.jobs.published_running_total) {
    const chartDom = document.getElementById("line-chart");
    if (chartDom) {
      chartDom.innerHTML = `<div>${MESSAGE_EMPTY_STATS}</div>`;
    }
  } else {
    renderLineChart(stats.jobs.published_running_total);
  }

  if (!stats.jobs.views_daily) {
    const chartDom = document.getElementById("bar-daily");
    if (chartDom) {
      chartDom.innerHTML = `<div>${MESSAGE_EMPTY_STATS}</div>`;
    }
  } else {
    renderBarDailyChart(stats.jobs.views_daily, stats.ts_now, stats.ts_one_month_ago);
  }

  if (!stats.jobs.views_monthly) {
    const chartDom = document.getElementById("bar-monthly");
    if (chartDom) {
      chartDom.innerHTML = `<div>${MESSAGE_EMPTY_STATS}</div>`;
    }
  } else {
    renderBarMonthlyChart(stats.jobs.views_monthly, stats.ts_now, stats.ts_two_years_ago);
  }
};
