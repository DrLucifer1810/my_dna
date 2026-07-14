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
    invoke('login_google').then((res) => {
      document.getElementById("status-text").innerText = res;
    }).catch(console.error);
  });

  document.getElementById("force-analyze").addEventListener("click", () => {
    document.getElementById("status-text").innerText = "Analyzing OS timeline... please wait.";
    invoke('force_analyze').then((score) => {
      document.getElementById("status-text").innerText = "Analysis Complete!";
      radarChart.data.datasets[0].data = [
        score.d1_context,
        score.d2_interaction,
        score.d3_customization,
        score.d4_efficiency,
        score.d5_security,
        score.d6_collaboration
      ];
      radarChart.update();
    }).catch((err) => {
      document.getElementById("status-text").innerText = "Error: " + err;
      console.error(err);
    });
  });
});
