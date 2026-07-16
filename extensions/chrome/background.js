let socket = null;
const MYDNA_PORT = 5050;

function connectToMyDNA() {
    socket = new WebSocket(`ws://127.0.0.1:${MYDNA_PORT}/ws/extension`);

    socket.onopen = () => {
        console.log("myDNA Chrome Extension Connected");
        socket.send(JSON.stringify({ type: "handshake", tool: "chrome" }));
    };

    socket.onmessage = (event) => {
        console.log("Message from myDNA Server:", event.data);
    };

    socket.onclose = () => {
        console.log("Disconnected from myDNA. Retrying in 10s...");
        setTimeout(connectToMyDNA, 10000);
    };

    socket.onerror = (error) => {
        console.error("WebSocket Error:", error);
    };
}

// Khởi tạo kết nối khi Service Worker bắt đầu
connectToMyDNA();

// Lắng nghe Message từ Content Script (khi user chat với AI)
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.type === "llm_interaction" && socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({
            ...request,
            url: sender.tab?.url
        }));
        sendResponse({ status: "sent" });
    }
});
