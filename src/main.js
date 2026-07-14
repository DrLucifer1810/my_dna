const { invoke } = window.__TAURI__.core;

let radarChart;

document.addEventListener("DOMContentLoaded", () => {
  // Tab Switching Logic
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

  // Radar Chart Initialization
  const ctx = document.getElementById("radar-chart").getContext("2d");
  radarChart = new Chart(ctx, {
    type: 'radar',
    data: {
      labels: ['Competence', 'Discipline', 'Creativity', 'Critical Thinking', 'Collaboration', 'AI Efficiency'],
      datasets: [{
        label: 'Enterprise Capability Matrix',
        data: [0, 0, 0, 0, 0, 0],
        backgroundColor: 'rgba(96, 165, 250, 0.2)',
        borderColor: 'rgba(96, 165, 250, 1)',
        pointBackgroundColor: 'rgba(96, 165, 250, 1)',
        pointBorderColor: '#fff',
        pointHoverBackgroundColor: '#fff',
        pointHoverBorderColor: 'rgba(96, 165, 250, 1)'
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
      plugins: { legend: { labels: { color: '#f8fafc' } } }
    }
  });

  // Fetch Metrics
  document.getElementById("refresh-chart").addEventListener("click", () => {
    invoke('get_evaluation_metrics').then((res) => {
      // Expecting array: [Competence, Discipline, Creativity, Critical Thinking, Collaboration, AI Efficiency]
      radarChart.data.datasets[0].data = res.metrics;
      radarChart.update();
    }).catch(console.error);
  });

  // Fetch Profile
  invoke('get_user_profile').then((res) => {
    if(res) {
      document.getElementById("user-profile-text").innerText = res;
    }
  }).catch(console.error);

  // Settings Toggles
  document.getElementById("login-google").addEventListener("click", () => {
    document.getElementById("settings-status").innerText = "Authenticating with Google...";
    invoke('login_google').then((res) => {
      document.getElementById("settings-status").innerText = res;
    }).catch(console.error);
  });

  document.getElementById("force-diagnostic").addEventListener("click", () => {
    document.getElementById("settings-status").innerText = "Forcing Profile Diagnostic... Please wait.";
    invoke('force_profile_diagnostic').then((res) => {
      document.getElementById("settings-status").innerText = "Diagnostic Complete!";
      document.getElementById("user-profile-text").innerText = res;
    }).catch(console.error);
  });
});
