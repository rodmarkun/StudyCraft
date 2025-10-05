<script>
  import { onMount, onDestroy } from "svelte";
  import { BarChart3 } from "lucide-svelte";
  import Chart from "chart.js/auto";

  // Props
  export let data = [];

  // State
  let chartCanvas;
  let chartInstance = null;
  let canvasId = `chart-${Math.random().toString(36).substr(2, 9)}`;

  // Reactive statement to recreate chart when data changes
  $: if (data && data.length > 0 && chartCanvas) {
    createChart();
  }

  // Functions
  function formatDuration(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);

    if (hours > 0) {
      return `${hours}h ${minutes}m`;
    } else if (minutes > 0) {
      return `${minutes}m`;
    } else {
      return `${seconds}s`;
    }
  }

  function createChart() {
    if (!chartCanvas || !data || data.length === 0) {
      console.log("Chart creation failed - missing data or canvas:", {
        hasCanvas: !!chartCanvas,
        dataLength: data?.length || 0,
      });
      return;
    }

    // Destroy existing chart first
    if (chartInstance) {
      chartInstance.destroy();
      chartInstance = null;
    }

    const ctx = chartCanvas.getContext("2d");

    const chartData = data.map((item) => item.studyTime);
    const hasData = chartData.some((value) => value > 0);

    chartInstance = new Chart(ctx, {
      type: "line",
      data: {
        labels: data.map((item) => item.dateLabel),
        datasets: [
          {
            label: "Study Time",
            data: chartData,
            borderColor: "#10b981",
            backgroundColor: (ctx) => {
              const chart = ctx.chart;
              const { ctx: canvasCtx, chartArea } = chart;
              if (!chartArea) {
                return "rgba(16, 185, 129, 0.1)";
              }
              const gradient = canvasCtx.createLinearGradient(
                0,
                chartArea.bottom,
                0,
                chartArea.top,
              );
              gradient.addColorStop(0, "rgba(16, 185, 129, 0.05)");
              gradient.addColorStop(1, "rgba(16, 185, 129, 0.3)");
              return gradient;
            },
            borderWidth: 3,
            pointBackgroundColor: data.map((item, index) => {
              if (item.studyTime === 0) return "transparent";

              const previousSessions = data
                .slice(0, index)
                .filter((d) => d.studyTime > 0);
              const isNewRecord =
                previousSessions.length === 0 ||
                item.studyTime >
                  Math.max(...previousSessions.map((d) => d.studyTime));

              return isNewRecord && item.studyTime > 0 ? "#f59e0b" : "#10b981"; // gold for records, green for normal
            }),
            pointBorderColor: "#fff",
            pointBorderWidth: 2,
            pointRadius: data.map((item) => (item.studyTime > 0 ? 6 : 0)),
            pointHoverRadius: data.map((item) => (item.studyTime > 0 ? 10 : 0)),
            fill: true,
            tension: 0.4,
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            display: false,
          },
          tooltip: {
            callbacks: {
              label: function (context) {
                const seconds = Number(context.parsed.y);
                return `Study Time: ${formatDuration(seconds)}`;
              },
              afterLabel: function (context) {
                const seconds = Number(context.parsed.y);
                if (seconds === 0) return "";

                const dataIndex = context.dataIndex;
                const dataset = context.dataset;
                // Extract numeric values from the dataset
                const allData = dataset.data.map((d) =>
                  typeof d === "number"
                    ? d
                    : typeof d === "object" && d !== null && "y" in d
                      ? Number(d.y)
                      : 0,
                );
                const previousData = allData
                  .slice(0, dataIndex)
                  .filter((d) => d > 0);
                const isNewRecord =
                  previousData.length === 0 ||
                  seconds > Math.max(...previousData);

                if (isNewRecord) return "🏆 New Record!";
                return "📊 Study Session";
              },
            },
            backgroundColor: "rgba(0, 0, 0, 0.9)",
            titleColor: "#fff",
            bodyColor: "#fff",
            borderColor: "#10b981",
            borderWidth: 2,
            cornerRadius: 8,
            padding: 12,
            titleFont: { size: 13, weight: "bold" },
            bodyFont: { size: 12 },
          },
        },
        scales: {
          x: {
            grid: {
              display: false,
            },
            ticks: {
              color: "#6b7280",
              font: { size: 11 },
              maxRotation: 0,
              callback: function (value, index, ticks) {
                // Show every 5th label to avoid overcrowding
                if (index % 5 === 0 || index === ticks.length - 1) {
                  const tickValue =
                    typeof value === "number" ? value : Number(value);
                  return this.getLabelForValue(tickValue);
                }
                return "";
              },
            },
          },
          y: {
            beginAtZero: true,
            grid: {
              color: "rgba(107, 114, 128, 0.1)",
              // Note: borderDash might not be available in all Chart.js versions
              // If you get an error, comment out the next line
              // borderDash: [2, 4]
            },
            ticks: {
              color: "#6b7280",
              font: { size: 11 },
              callback: function (value) {
                const numValue = Number(value);
                if (numValue === 0) return "0s";
                return formatDuration(numValue);
              },
            },
          },
        },
        animation: {
          duration: 2000,
          easing: "easeOutCubic",
        },
        interaction: {
          intersect: false,
          mode: "index",
        },
        elements: {
          point: {
            hoverBackgroundColor: function (context) {
              const value = Number(context.parsed.y);
              if (value === 0) return "transparent";

              const dataIndex = context.dataIndex;
              const dataset = context.dataset;
              // Extract numeric values from the dataset
              const allData = dataset.data.map((d) =>
                typeof d === "number"
                  ? d
                  : typeof d === "object" && d !== null && "y" in d
                    ? Number(d.y)
                    : 0,
              );
              const previousData = allData
                .slice(0, dataIndex)
                .filter((d) => d > 0);
              const isNewRecord =
                previousData.length === 0 || value > Math.max(...previousData);

              return isNewRecord ? "#f59e0b" : "#10b981";
            },
          },
        },
      },
    });
  }

  onMount(() => {
    console.log(
      "StudyTimeChart mounted with data:",
      data?.length || 0,
      "points",
    );
    setTimeout(() => {
      if (data && data.length > 0) {
        createChart();
      } else {
        console.log("No data available for initial chart");
      }
    }, 100);
  });

  onDestroy(() => {
    if (chartInstance) {
      chartInstance.destroy();
      chartInstance = null;
    }
  });
</script>

<div class="sv-chart-wrapper">
  {#if data && data.length > 0}
    <canvas bind:this={chartCanvas} id={canvasId} class="sv-chart-canvas"
    ></canvas>

    <div class="sv-study-legend">
      <div class="sv-legend-item">
        <div class="sv-legend-dot" style="background: #10b981;"></div>
        <span>Study Session</span>
      </div>
      <div class="sv-legend-item">
        <div class="sv-legend-dot" style="background: #f59e0b;"></div>
        <span>New Record</span>
      </div>
    </div>
  {:else}
    <div class="sv-chart-empty">
      <div class="sv-chart-empty-icon">
        <BarChart3 size={48} />
      </div>
      <h4>No Study Data Yet</h4>
      <p>Start studying to see your daily progress!</p>
    </div>
  {/if}
</div>

<style>
  .sv-chart-wrapper {
    height: 300px;
    width: 100%;
    position: relative;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
    padding: 1.5rem;
    padding-bottom: 3rem;
  }

  .sv-chart-canvas {
    width: 100%;
    height: calc(100% - 40px);
    border-radius: 8px;
  }

  .sv-study-legend {
    position: absolute;
    bottom: 1rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 1.5rem;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }

  .sv-legend-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .sv-legend-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .sv-chart-empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    color: var(--text-secondary);
    padding: 2rem;
  }

  .sv-chart-empty-icon {
    font-size: 3rem;
    margin-bottom: 1rem;
    opacity: 0.6;
    animation: sv-float 3s ease-in-out infinite;
  }

  @keyframes sv-float {
    0%,
    100% {
      transform: translateY(0px);
    }
    50% {
      transform: translateY(-8px);
    }
  }

  .sv-chart-empty h4 {
    margin: 0 0 0.5rem 0;
    color: var(--text);
    font-size: 1.1rem;
    font-weight: 500;
  }

  .sv-chart-empty p {
    margin: 0;
    font-size: 0.875rem;
    opacity: 0.8;
  }

  @media (max-width: 768px) {
    .sv-chart-wrapper {
      height: 250px;
      padding: 1rem;
      padding-bottom: 2.5rem;
    }

    .sv-study-legend {
      flex-wrap: wrap;
      gap: 1rem;
      bottom: 0.5rem;
    }

    .sv-legend-item {
      font-size: 0.7rem;
    }
  }
</style>
