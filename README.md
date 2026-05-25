# The Signal 🛰️

An AI-powered elite tech curator built for senior backend engineers. This project is a production-ready MVP designed and developed for the **#dailydevhackathon**.

The Signal solves information overload by filtering out beginner-friendly tutorials, community fluff, and tech anniversaries from daily feeds. It delivers only the top 3 high-impact architectural and performance-driven articles based on your custom scope.

## 🛠️ Tech Stack
- **Language:** Rust 🦀
- **Web Framework:** Axum
- **Async Runtime:** Tokio
- **HTTP Client:** Reqwest
- **AI Engine:** Gemini AI (`gemini-2.5-flash`)

## 🚀 Features
- **Dynamic Scoping:** Query parameters for custom topics (`theme`) and timelines (`time` e.g., `day`, `week`, `month`, `year`, `all`).
- **AI-Driven Curation:** Discards the noise and distills dozens of upstream posts down to the top 3 high-impact architectural reads.
- **Clean JSON Output:** Structured precisely for seamless frontend integration or CLI automation.

## ⚙️ Installation & Local Setup

### 1. Clone the Repository
git clone [https://github.com/YOUR_USERNAME/the-signal-backend.git](https://github.com/YOUR_USERNAME/the-signal-backend.git)
cd the-signal-backend

### 2. Configure Environment Variables
The application reads keys directly from your system environment to prevent credential leaks. Run the server by passing your tokens directly in your terminal execution:

DAILY_DEV_TOKEN="your_daily_dev_token" GEMINI_API_KEY="your_gemini_api_key" cargo run

The MVP API endpoint will be active and listening at http://localhost:8000.

## 🧪 Quick API Testing
You can test the endpoint using curl or any HTTP client:

GET http://localhost:8000/api/signal?theme=rust&time=week

### Example JSON Response
[
  {
    "title": "Rust Untrusted type proposal could eliminate 80% of Linux kernel CVEs",
    "url": "[https://feed.itsfoss.com/link/24361/17345650/linux-kernel-rust-cve-reduction](https://feed.itsfoss.com/link/24361/17345650/linux-kernel-rust-cve-reduction)",
    "reason": "The compile-time `Untrusted<T>` type proposal fundamentally eliminates the majority of memory safety vulnerabilities at the Linux kernel level without runtime overhead."
  }
]

## 🗺️ Future Roadmaps (Scaling the MVP)
- **In-Memory Caching:** Store curated feeds temporarily to guarantee sub-second HTTP responses and completely bypass upstream rate limits.
- **Background Cron Workers:** Shift the heavy ingestion pipeline to a background task runner, ensuring instant data delivery and zero client request timeouts.

---
Built with passion for the **#dailydevhackathon** powered by @dailydotdev. Let's keep our tech feeds high-signal! ⚡
