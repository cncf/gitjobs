import { getBarStatsOptions, gitjobsChartTheme } from "/static/js/jobboard/stats.js";
import { prettifyNumber, toggleModalVisibility } from "/static/js/common/common.js";
import { showErrorAlert } from "/static/js/common/alerts.js";

/**
 * Function to render the statistics chart for a job
 * @param {string} id - The ID of the job to render stats for
 * @private
 */
const renderStat = (data) => {
  const today = Date.now();
  // Set the minimum date to one month ago
  const min = new Date();
  const month = min.getMonth();
  min.setMonth(min.getMonth() - 1);

  // If today is the first day of the month, set it to the last day of
  // the previous month
  if (min.getMonth() == month) min.setDate(0);
  // Set the time to the start of the day
  min.setHours(0, 0, 0, 0);

  const chartDom = document.getElementById(`job-stats`);
  if (!chartDom) return;

  const myChart = echarts.init(chartDom, "gitjobs", {
    renderer: "svg",
    useDirtyRect: false,
  });
  myChart.clear();

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
      min: min,
      max: today,
    },
  };

  option && myChart.setOption(option);
};

/**
 * Fetches and renders statistics for a specific job
 * @param {string} id - The ID of the job to fetch stats for
 */
export const fetchStats = async (id) => {
  const response = await fetch(`/dashboard/employer/jobs/${id}/stats`, {
    method: "GET",
  });
  const spinnerStats = document.getElementById(`spinner-stats-${id}`);
  if (!response.ok) {
    // Hide the spinner if it exists
    if (spinnerStats) {
      spinnerStats.classList.add("hidden");
    }

    showErrorAlert("Something went wrong fetching the stats, please try again later.");
    return;
  }
  const data = await response.json();
  // Hide the spinner if it exists
  if (spinnerStats) {
    spinnerStats.classList.add("hidden");
  }
  if (data) {
    toggleModalVisibility(`stats-modal`, "open");
    if (data.views_daily.length > 0) {
      renderStat(data.views_daily);
    } else {
      const chartDom = document.getElementById(`job-stats`);
      if (chartDom) {
        chartDom.textContent = "No data available on last month";
      }
    }
    if (data.views_total_last_month !== undefined) {
      const totalViewsElement = document.getElementById(`total-views`);
      if (totalViewsElement) {
        totalViewsElement.textContent = prettifyNumber(data.views_total_last_month);
      }
    }
  }
};

/**
 * Closes the stats modal and clears its content
 */
export const closeStatsModal = () => {
  const chartDom = document.getElementById(`job-stats`);
  if (chartDom) {
    const chartInstance = echarts.getInstanceByDom(chartDom);
    // Dispose of the chart instance before closing modal
    if (chartInstance) {
      chartInstance.dispose();
      // Clear the chart content
    } else {
      chartDom.textContent = "";
    }
  }
  // Close the stats modal
  toggleModalVisibility(`stats-modal`, "close");
  // Clear the total views element
  const totalViewsElement = document.getElementById(`total-views`);
  if (totalViewsElement) {
    totalViewsElement.textContent = "";
  }
};

/**
 * Registers the GitJobs theme for ECharts
 */
export const registerEchartsTheme = () => {
  // Register the GitJobs theme
  echarts.registerTheme("gitjobs", gitjobsChartTheme);
};
