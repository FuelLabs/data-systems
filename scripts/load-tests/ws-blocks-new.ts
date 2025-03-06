import ws from "k6/ws";
import { check, sleep } from "k6";
import { Counter } from "k6/metrics";

export const options = {
    stages: [
        { duration: "1m", target: 100 },
        { duration: "2m", target: 100 },
        { duration: "1m", target: 500 },
        { duration: "2m", target: 500 },
        { duration: "1m", target: 0 },
    ],
    thresholds: {
        Errors: ["count < 10"],
        ws_connecting: ["p(90) < 500"], // 90% of connections under 500ms
        ws_msgs_received: ["rate >= 500"], // At least 500 messages/second at peak (1 per VU)
        ws_msgs_sent: ["rate > 0"], // Ensure some messages are sent
        ws_session_duration: ["p(90) > 10000"], // 90% of sessions last at least 10s
        ws_sessions: ["count >= 500"], // At least 500 sessions started
        checks: ["rate > 0.95"], // 95% of checks pass
    },
};

function createSubscriptionPayload(subject = "blocks", params = {}) {
    return JSON.stringify({
        deliverPolicy: "new",
        subscribe: [{ subject: subject, params: params }],
    });
}

const API_KEY = "fuel-CPEYX4t94gijC_q2b.U6AXNENm-Uqnhw";
export const WsErrors = new Counter("Errors");

export default function () {
    const url = `wss://stream-mainnet.fuel.network/api/v1/ws?api_key=${API_KEY}`;

    const res = ws.connect(url, {}, (socket) => {
        socket.on("open", () => {
            const subPayload = createSubscriptionPayload();
            socket.send(subPayload);
        });

        socket.on("message", (data) => {
            const message = JSON.parse(data);
            check(message, {
                "no error in message": (msg) => !msg.error,
                "subscription successful": (msg) => msg.response?.subject?.includes("blocks"),
            });
        });

        socket.on("error", (e) => {
            console.error("WebSocket error:", e);
            WsErrors.add(1);
        });

        socket.setTimeout(() => {
            socket.close();
        }, 30000); // 30s session
    });

    check(res, { "status is 101": (r) => r && r.status === 101 });
    sleep(1);
}
