import * as vscode from 'vscode';
import * as WebSocket from 'ws';

let ws: WebSocket | null = null;
const MYDNA_PORT = 5050;

export function activate(context: vscode.ExtensionContext) {
    console.log('myDNA Telemetry Agent is now active!');

    // Lệnh kết nối thủ công
    let connectCmd = vscode.commands.registerCommand('mydna.connect', () => {
        connectToMyDNA();
    });
    context.subscriptions.push(connectCmd);

    // Tự động kết nối khi khởi động
    connectToMyDNA();

    // Theo dõi Text Document thay đổi
    vscode.workspace.onDidChangeTextDocument((event) => {
        if (ws && ws.readyState === WebSocket.OPEN) {
            const fileName = event.document.fileName;
            const changes = event.contentChanges;
            
            // Lọc các thao tác Paste lớn (Khả năng là copy từ AI)
            const pastedText = changes.find(c => c.text.length > 50);
            if (pastedText) {
                ws.send(JSON.stringify({
                    type: "llm_interaction",
                    tool: "vscode",
                    action: "code_pasted",
                    file: fileName,
                    content: pastedText.text
                }));
            }
        }
    });

    // TODO: Tương lai sẽ tích hợp API của vscode.chat (Copilot API) để lắng nghe Prompt
}

function connectToMyDNA() {
    if (ws && ws.readyState === WebSocket.OPEN) {
        return;
    }

    vscode.window.showInformationMessage('Connecting to myDNA Local Server...');
    ws = new WebSocket(`ws://127.0.0.1:${MYDNA_PORT}/ws/extension`);

    ws.on('open', () => {
        vscode.window.showInformationMessage('myDNA: Connected securely to Local Server.');
        ws?.send(JSON.stringify({ type: "handshake", tool: "vscode" }));
    });

    ws.on('error', (err) => {
        console.error('myDNA Connection Error:', err);
    });

    ws.on('close', () => {
        console.log('myDNA Connection Closed. Retrying in 10s...');
        setTimeout(connectToMyDNA, 10000);
    });
}

export function deactivate() {
    if (ws) {
        ws.close();
    }
}
