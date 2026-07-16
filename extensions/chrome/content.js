// Đây là bản khung (Boilerplate)
// Sau này sẽ bổ sung các bộ chọn DOM (DOM Selectors) chính xác cho ChatGPT, Claude, Gemini

console.log("myDNA Web Watcher is observing this AI chat...");

// Ví dụ: Bắt sự kiện Copy code từ ChatGPT
document.addEventListener("copy", (e) => {
    const selection = document.getSelection()?.toString();
    if (selection && selection.length > 20) {
        chrome.runtime.sendMessage({
            type: "llm_interaction",
            tool: window.location.hostname,
            action: "code_copied",
            content: selection
        });
    }
});

// Ví dụ: Bắt sự kiện Gửi form (User Prompt)
// Sẽ cài đặt MutationObserver để phát hiện khi AI trả lời xong ở phiên bản sau.
