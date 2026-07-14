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
  const tabP2p = document.getElementById("tab-p2p");
  const viewDashboard = document.getElementById("view-dashboard");
  const viewSettings = document.getElementById("view-settings");
  const viewP2p = document.getElementById("view-p2p");

  function switchTab(activeTab, activeView) {
    [tabDashboard, tabSettings, tabP2p].forEach(t => t.classList.remove("active"));
    [viewDashboard, viewSettings, viewP2p].forEach(v => v.style.display = "none");
    
    activeTab.classList.add("active");
    activeView.style.display = "block";
  }

  tabDashboard.addEventListener("click", () => switchTab(tabDashboard, viewDashboard));
  tabSettings.addEventListener("click", () => switchTab(tabSettings, viewSettings));
  tabP2p.addEventListener("click", () => switchTab(tabP2p, viewP2p));

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

  const btnSync = document.getElementById("sync-google-drive");
  if(btnSync) {
    btnSync.addEventListener("click", () => {
      document.getElementById("settings-status").innerText = "Đang mở luồng Google OAuth2 & Đồng bộ...";
      invoke('login_and_sync_google_drive').then((res) => {
        document.getElementById("settings-status").innerText = res;
      }).catch(err => {
        document.getElementById("settings-status").innerHTML = `<span style="color:#ef4444;">Lỗi Đồng bộ: ${err}</span>`;
      });
    });
  }

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

  const p2pBtn = document.getElementById("start-p2p-btn");
  if(p2pBtn) {
    p2pBtn.addEventListener("click", () => {
        const intentRecruiting = document.getElementById("intent-recruiting").checked;
        const intentLookingJob = document.getElementById("intent-looking-job").checked;
        const intentHiringFreelancer = document.getElementById("intent-hiring-freelancer").checked;
        const intentFreelancing = document.getElementById("intent-freelancing").checked;
        const contactEmail = document.getElementById("p2p-contact-email").value.trim();

        document.getElementById("p2p-status").innerText = "Đang khởi động Node P2P...";
        invoke('start_p2p_network', {
            intentRecruiting,
            intentLookingJob,
            intentHiringFreelancer,
            intentFreelancing,
            contactEmail
        }).then((res) => {
            document.getElementById("p2p-status").innerText = res;
        }).catch(err => {
            document.getElementById("p2p-status").innerHTML = `<span style="color:#ef4444;">Lỗi: ${err}</span>`;
        });
    });
  }
});
