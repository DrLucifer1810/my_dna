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
  renderChart();

  // Fetch Metrics
  document.getElementById("refresh-chart").addEventListener("click", () => {
    renderChart();
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
