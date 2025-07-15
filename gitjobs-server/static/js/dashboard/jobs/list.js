import { getBarStatsOptions, gitjobsChartTheme } from "/static/js/jobboard/stats.js";
import { prettifyNumber, toggleModalVisibility } from "/static/js/common/common.js";
import { showErrorAlert, showInfoAlert } from "/static/js/common/alerts.js";

/**
 * Shows statistics for a specific job in a modal
 * @param {string} id - The ID of the job to display stats for
 */
export const showStats = async (id) => {
  // Get loading spinner reference
  const spinnerStats = document.getElementById(`spinner-stats-${id}`);

  // Fetch job statistics from the API
  const response = await fetch(`/dashboard/employer/jobs/${id}/stats`, {
    method: "GET",
  });
  if (!response.ok) {
    if (spinnerStats) {
      spinnerStats.classList.add("hidden");
    }
    showErrorAlert("Something went wrong fetching the stats, please try again later.");
    return;
  }
  const data = await response.json();
  if (spinnerStats) {
    spinnerStats.classList.add("hidden");
  }

  // Process and display the statistics data
  if (data) {
    const hasViewsData = data.views_daily && data.views_daily.length > 0;
    const hasSearchAppearancesData =
      data.search_appearances_daily && data.search_appearances_daily.length > 0;

    if (hasViewsData || hasSearchAppearancesData) {
      // Open the statistics modal if we have data for at least one chart
      toggleModalVisibility(`stats-modal`, "open");

      // Render views chart if data exists
      if (hasViewsData) {
        renderChart(data.views_daily, "job-chart-views", "views");
        if (data.views_total_last_month !== undefined) {
          const totalViewsElement = document.getElementById("total-views");
          if (totalViewsElement) {
            totalViewsElement.textContent = prettifyNumber(data.views_total_last_month);
          }
        }
      } else {
        // Hide views chart if no data is available
        const viewsChartWrapper = document.querySelector('[data-chart="views"]');
        if (viewsChartWrapper) {
          viewsChartWrapper.classList.add("hidden");
        }
      }

      // Render search appearances chart if data exists
      if (hasSearchAppearancesData) {
        renderChart(data.search_appearances_daily, "job-chart-search-appearances", "search_appearances");
        if (data.search_appearances_total_last_month !== undefined) {
          const totalSearchElement = document.getElementById("total-search-appearances");
          if (totalSearchElement) {
            totalSearchElement.textContent = prettifyNumber(data.search_appearances_total_last_month);
          }
        }
      } else {
        // Hide search appearances chart if no data is available
        const searchAppearancesChartWrapper = document.querySelector('[data-chart="search-appearances"]');
        if (searchAppearancesChartWrapper) {
          searchAppearancesChartWrapper.classList.add("hidden");
        }
      }
    } else {
      // Show message when no data is available for either chart
      showInfoAlert(
        'We don\'t have statistics data for this job yet.<div class="mt-2">Please check again later.</div>',
        true,
      );
    }
  }
};

/**
 * Closes the statistics modal and cleans up resources
 */
export const closeStats = () => {
  // Dispose of all chart instances to free up memory
  const chartIds = ["job-chart-views", "job-chart-search-appearances"];
  chartIds.forEach((id) => {
    const chartDom = document.getElementById(id);
    if (chartDom) {
      const chartInstance = echarts.getInstanceByDom(chartDom);
      if (chartInstance) {
        chartInstance.dispose();
      }
    }
  });

  // Close the modal
  toggleModalVisibility(`stats-modal`, "close");

  // Clear the statistics counters
  const totalViewsElement = document.getElementById(`total-views`);
  if (totalViewsElement) {
    totalViewsElement.textContent = "";
  }
  const totalSearchElement = document.getElementById(`total-search-appearances`);
  if (totalSearchElement) {
    totalSearchElement.textContent = "";
  }

  // Display charts wrapper
  const viewsChartWrapper = document.querySelector('[data-chart="views"]');
  if (viewsChartWrapper) {
    viewsChartWrapper.classList.remove("hidden");
  }
  const searchAppearancesChartWrapper = document.querySelector('[data-chart="search-appearances"]');
  if (searchAppearancesChartWrapper) {
    searchAppearancesChartWrapper.classList.remove("hidden");
  }
};

/**
 * Function to render a chart
 * @param {Array} data - The chart data
 * @param {string} chartId - The ID of the chart container
 * @param {string} chartType - The type of chart ('views' or 'search_appearances')
 * @private
 */
const renderChart = (data, chartId, chartType) => {
  // Calculate date range for the chart (last 30 days)
  const today = Date.now();
  const min = new Date();
  const month = min.getMonth();
  min.setMonth(min.getMonth() - 1);
  // Handle edge case when today is the first day of the month
  if (min.getMonth() == month) min.setDate(0);
  min.setHours(0, 0, 0, 0);

  // Get the chart container element
  const chartDom = document.getElementById(chartId);
  if (!chartDom) return;

  // Initialize the ECharts instance
  const chart = echarts.init(chartDom, "gitjobs", {
    renderer: "svg",
    useDirtyRect: false,
  });
  chart.clear();

  // Add responsive resize handler
  window.addEventListener("resize", function () {
    chart.resize();
  });

  // Configure chart options
  const option = {
    ...getBarStatsOptions(),
    dataset: [
      {
        dimensions: ["timestamp", "count"],
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
        const label = chartType === "views" ? "Views" : "Search appearances";
        return `${chartdate}<br />${label}: ${prettifyNumber(params.data[1])}`;
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

  // Render the chart with the configured options
  option && chart.setOption(option);
};

/**
 * Registers the GitJobs theme for ECharts
 */
export const registerEchartsTheme = () => {
  // Register the custom GitJobs theme for consistent chart styling
  echarts.registerTheme("gitjobs", gitjobsChartTheme);
};
