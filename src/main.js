const { invoke } = window.__TAURI__.core;

let radarChart;

document.addEventListener("DOMContentLoaded", () => {
  const ctx = document.createElement("canvas");
  document.getElementById("radar-chart").innerHTML = "";
  document.getElementById("radar-chart").appendChild(ctx);

  radarChart = new Chart(ctx, {
    type: 'radar',
    data: {
      labels: ['D1 Context', 'D2 Interaction', 'D3 Customization', 'D4 Efficiency', 'D5 Security', 'D6 Collaboration'],
      datasets: [{
        label: 'AI Competency DNA',
        data: [0, 0, 0, 0, 0, 0], // Dữ liệu sẽ được load từ Rust Backend
        backgroundColor: 'rgba(54, 162, 235, 0.2)',
        borderColor: 'rgb(54, 162, 235)',
        pointBackgroundColor: 'rgb(54, 162, 235)',
        pointBorderColor: '#fff',
        pointHoverBackgroundColor: '#fff',
        pointHoverBorderColor: 'rgb(54, 162, 235)'
      }]
    },
    options: {
      elements: {
        line: { borderWidth: 3 }
      },
      scales: {
        r: {
          angleLines: { display: false },
          suggestedMin: 0,
          suggestedMax: 100
        }
      }
    },
  });

  document.getElementById("login-google").addEventListener("click", () => {
    document.getElementById("status-text").innerText = "Authenticating with Google...";
    // Gọi Tauri Command để chạy luồng OAuth2
    // invoke('google_login')
  });
});
