const { invoke } = window.__TAURI__.core;

let radarChart;

async function renderChart() {
  const statusEl = document.getElementById("chart-status");
  const canvasEl = document.getElementById("radarChart");
  
  statusEl.innerText = "Đang tải dữ liệu Radar...";
  statusEl.style.display = "block";
  canvasEl.style.display = "none";

  try {
    const res = await invoke('get_evaluation_metrics');
    if (res && res.metrics) {
      statusEl.style.display = "none";
      canvasEl.style.display = "block";
      
      const ctx = canvasEl.getContext('2d');
      if (radarChart) {
        radarChart.destroy();
      }
      
      radarChart = new Chart(ctx, {
        type: 'radar',
        data: {
          labels: ['Competence', 'Discipline', 'Creativity', 'Critical Thinking', 'Collaboration', 'AI Efficiency'],
          datasets: [{
            label: 'Current Session Performance',
            data: res.metrics,
            backgroundColor: 'rgba(37, 99, 235, 0.2)',
            borderColor: 'rgba(37, 99, 235, 1)',
            pointBackgroundColor: 'rgba(37, 99, 235, 1)',
            borderWidth: 2
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          scales: {
            r: {
              angleLines: { color: 'rgba(255, 255, 255, 0.1)' },
              grid: { color: 'rgba(255, 255, 255, 0.1)' },
              pointLabels: { color: '#94a3b8', font: { size: 12 } },
              ticks: { display: false, min: 0, max: 100 }
            }
          },
          plugins: {
            legend: { labels: { color: '#f8fafc' } }
          }
        }
      });
    }
  } catch (err) {
    console.error("Radar Error:", err);
    statusEl.innerHTML = `<span style="color: #ef4444;">❌ ${err}</span>`;
  }
}

async function loadDNAProfile() {
  const dnaStatus = document.getElementById("dna-status");
  const dnaCards = document.querySelectorAll(".dna-card");
  
  dnaStatus.innerText = "Đang tải hồ sơ DNA...";
  dnaStatus.style.display = "block";
  dnaCards.forEach(c => c.style.display = "none");

  try {
    const dna = await invoke('get_dna_profile');
    if (dna) {
      dnaStatus.style.display = "none";
      dnaCards.forEach(c => c.style.display = "block");
      
      document.getElementById("dna-profession").innerText = dna.profession || "N/A";
      document.getElementById("dna-focus").innerText = dna.daily_focus || "N/A";
      
      const goodHabits = dna.coding_habits?.good || [];
      const badHabits = dna.coding_habits?.bad || [];
      document.getElementById("dna-good-habits").innerText = goodHabits.join(", ") || "N/A";
      document.getElementById("dna-bad-habits").innerText = badHabits.join(", ") || "N/A";
      
      const tone = dna.tone || [];
      document.getElementById("dna-tone").innerText = tone.join(", ") || "N/A";
    }
  } catch (err) {
    console.error("DNA Profile Error:", err);
    dnaStatus.innerHTML = `<span style="color: #ef4444;">❌ ${err}</span>`;
  }
}

document.addEventListener("DOMContentLoaded", () => {
  const tabDashboard = document.getElementById("tab-dashboard");
  const tabSettings = document.getElementById("tab-settings");
  const viewDashboard = document.getElementById("view-dashboard");
  const viewSettings = document.getElementById("view-settings");

  tabDashboard.addEventListener("click", () => {
    tabDashboard.classList.add("active");
    tabSettings.classList.remove("active");
    viewDashboard.style.display = "block";
    viewSettings.style.display = "none";
  });

  tabSettings.addEventListener("click", () => {
    tabSettings.classList.add("active");
    tabDashboard.classList.remove("active");
    viewSettings.style.display = "block";
    viewDashboard.style.display = "none";
  });

  // Initial loads
  renderChart();
  loadDNAProfile();

  document.getElementById("refresh-chart").addEventListener("click", () => {
    renderChart();
    loadDNAProfile();
  });

  document.getElementById("login-google").addEventListener("click", () => {
    document.getElementById("settings-status").innerText = "Authenticating with Google...";
    invoke('login_google').then((res) => {
      document.getElementById("settings-status").innerText = res;
    }).catch(err => {
      document.getElementById("settings-status").innerHTML = `<span style="color:#ef4444;">Error: ${err}</span>`;
    });
  });

  const forceBtn = document.getElementById("force-diagnostic");
  if(forceBtn) {
      forceBtn.addEventListener("click", () => {
        document.getElementById("settings-status").innerText = "Forcing Log Analysis... Please wait.";
        invoke('force_analyze_logs').then(() => {
          document.getElementById("settings-status").innerText = "Analysis Complete! Refreshing dashboard...";
          renderChart();
          loadDNAProfile();
        }).catch(err => {
          document.getElementById("settings-status").innerHTML = `<span style="color:#ef4444;">Error: ${err}</span>`;
        });
      });
  }
});
