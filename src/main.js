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
  const tabIntegrations = document.getElementById("tab-integrations");
  const viewDashboard = document.getElementById("view-dashboard");
  const viewSettings = document.getElementById("view-settings");
  const viewP2p = document.getElementById("view-p2p");
  const viewIntegrations = document.getElementById("view-integrations");

  function switchTab(activeTab, activeView) {
    [tabDashboard, tabSettings, tabP2p, tabIntegrations].forEach(t => t.classList.remove("active"));
    [viewDashboard, viewSettings, viewP2p, viewIntegrations].forEach(v => v.style.display = "none");
    
    activeTab.classList.add("active");
    activeView.style.display = "block";
  }

  tabDashboard.addEventListener("click", () => switchTab(tabDashboard, viewDashboard));
  tabSettings.addEventListener("click", () => switchTab(tabSettings, viewSettings));
  tabP2p.addEventListener("click", () => switchTab(tabP2p, viewP2p));
  tabIntegrations.addEventListener("click", () => switchTab(tabIntegrations, viewIntegrations));

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
        const matchingProfileRaw = document.getElementById("matching-profile-data").value;
        const matchingProfileJson = matchingProfileRaw ? matchingProfileRaw : null;

        document.getElementById("p2p-status").innerText = "Đang khởi động Node P2P...";
        invoke('start_p2p_network', {
            intentRecruiting,
            intentLookingJob,
            intentHiringFreelancer,
            intentFreelancing,
            contactEmail,
            matchingProfileJson
        }).then((msg) => {
            document.getElementById("p2p-status").innerHTML = `<span style="color:#10b981;">✅ ${msg}</span>`;
        }).catch((err) => {
            document.getElementById("p2p-status").innerHTML = `<span style="color:#ef4444;">❌ Lỗi: ${err}</span>`;
        });
    });
  }

  document.getElementById("parse-ai-btn").addEventListener("click", () => {
      const text = document.getElementById("ai-context-input").value.trim();
      if (!text) {
          alert("Vui lòng nhập nội dung JD hoặc CV!");
          return;
      }

      const isRecruiting = document.getElementById("intent-recruiting").checked || document.getElementById("intent-hiring-freelancer").checked;
      
      const cmd = isRecruiting ? 'parse_jd_to_profile' : 'parse_cv_to_profile';
      const argName = isRecruiting ? 'jdText' : 'cvText';
      const statusEl = document.getElementById("ai-parse-status");
      
      statusEl.innerText = "⏳ AI đang phân tích dữ liệu...";
      
      const args = {};
      args[argName] = text;
      
      invoke(cmd, args).then((jsonStr) => {
          document.getElementById("matching-profile-data").value = jsonStr;
          statusEl.innerHTML = `<span style="color:#10b981;">✅ Đã trích xuất Trọng số thành công!</span>`;
          console.log("Matching Profile:", JSON.parse(jsonStr));
      }).catch((err) => {
          statusEl.innerHTML = `<span style="color:#ef4444;">❌ Lỗi AI: ${err}</span>`;
      });
  });

  // --- INTEGRATION HUB ---
  const logMcpStatus = (msg, isError=false) => {
    const el = document.getElementById("mcp-status-log");
    el.innerHTML = isError ? `<span style="color:#ef4444;">❌ ${msg}</span>` : `✅ ${msg}`;
  };

  // 1-Click Installs
  document.getElementById("install-vscode-btn")?.addEventListener("click", () => {
    logMcpStatus("Đang cài đặt VS Code Plugin qua CLI...");
    invoke("install_vscode_extension").then(res => {
      logMcpStatus(res);
    }).catch(err => {
      logMcpStatus(err, true);
    });
  });

  document.getElementById("install-chrome-btn")?.addEventListener("click", () => {
    // Thông thường mở URL Store, ở đây gọi rust để open URL
    invoke("open_chrome_extension_store").then(res => {
      logMcpStatus(res);
    }).catch(err => logMcpStatus(err, true));
  });

  // MCP Connections
  const setupMcpButton = (btnId, inputId, serverName) => {
    document.getElementById(btnId)?.addEventListener("click", () => {
      const token = document.getElementById(inputId).value.trim();
      if (!token) {
        logMcpStatus(`Vui lòng nhập Token cho ${serverName}`, true);
        return;
      }
      logMcpStatus(`Đang kết nối MCP Server: ${serverName}...`);
      invoke("connect_mcp_server", { serverName, token }).then(res => {
        logMcpStatus(`MCP [${serverName}]: ${res}`);
      }).catch(err => logMcpStatus(`Lỗi MCP [${serverName}]: ${err}`, true));
    });
  };

  setupMcpButton("mcp-github-btn", "mcp-github-token", "github");
  setupMcpButton("mcp-jira-btn", "mcp-jira-token", "jira");
  setupMcpButton("mcp-slack-btn", "mcp-slack-token", "slack");
  setupMcpButton("mcp-notion-btn", "mcp-notion-token", "notion");

  // Autonomous Agents Auto-Discovery
  const scanAutonomousAgents = () => {
    const agents = ["antigravity", "claude", "openclaw", "cline"];
    agents.forEach(agent => {
        const el = document.getElementById(`agent-${agent}-status`);
        if (el) {
            el.innerText = "Đang quét...";
            el.style.color = "#94a3b8";
        }
    });

    invoke("check_autonomous_agents").then((res) => {
        // res là object: { antigravity: true, claude: false, openclaw: true, cline: true }
        agents.forEach(agent => {
            const el = document.getElementById(`agent-${agent}-status`);
            if (el) {
                if (res[agent]) {
                    el.innerHTML = `✅ Đã kết nối (Giám sát ngầm)`;
                    el.style.color = "#10b981";
                } else {
                    el.innerHTML = `❌ Không tìm thấy`;
                    el.style.color = "#ef4444";
                }
            }
        });
    }).catch(err => console.error("Error scanning agents:", err));
  };

  document.getElementById("scan-agents-btn")?.addEventListener("click", scanAutonomousAgents);
  
  // Quét tự động ngay khi tải trang
  scanAutonomousAgents();

  // Auto-Updater Check
  const checkUpdate = () => {
    invoke('check_for_updates').then((res) => {
      if (res && res.available) {
        document.getElementById('update-version').innerText = res.version || '';
        document.getElementById('update-notes').innerHTML = res.body ? res.body.replace(/\n/g, '<br>') : 'Không có thông tin chi tiết.';
        document.getElementById('update-modal').style.display = 'flex';
      }
    }).catch(err => console.error("Error checking for updates:", err));
  };
  
  document.getElementById('update-confirm-btn')?.addEventListener("click", () => {
    const btn = document.getElementById('update-confirm-btn');
    btn.innerText = "Đang tải và Cài đặt (Vui lòng chờ)...";
    btn.disabled = true;
    invoke('install_update').then(() => {
        btn.innerText = "Đã cập nhật (Khởi động lại...)";
    }).catch(err => {
        alert("Lỗi cập nhật: " + err);
        btn.innerText = "Cập nhật ngay";
        btn.disabled = false;
    });
  });

  // Automatically check for updates on startup
  setTimeout(checkUpdate, 3000);
});
