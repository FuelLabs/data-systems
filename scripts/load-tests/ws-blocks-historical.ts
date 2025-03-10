import ws from "k6/ws";
import { check, sleep } from "k6";
import { Counter } from "k6/metrics";
import { schema } from "./schema.ts";

export const options = {
    stages: [
        { duration: "2m", target: 5000 },
        { duration: "1m", target: 0 },
    ],
    thresholds: {
        Errors: ["count < 10"],
        ws_connecting: ["p(90) < 500"], // 90% of connections under 500ms
        ws_msgs_received: ["rate >= 300"], // At least 500 messages/second at peak (1 per VU)
        ws_msgs_sent: ["rate > 0"], // Ensure some messages are sent
        ws_session_duration: ["p(90) > 10000"], // 90% of sessions last at least 10s
        ws_sessions: ["count >= 500"], // At least 500 sessions started
        checks: ["rate > 0.95"], // 95% of checks pass
    },
};

function getRandomSubject() {
    const subjects: string[] = [];
    for (const [_, entry] of Object.entries(schema)) {
        subjects.push(entry.id);
        if ("variants" in entry) {
            for (const variantKey in entry.variants) {
                subjects.push(entry.variants[variantKey].id);
            }
        }
    }

    // Return a random subject from the list
    return subjects[Math.floor(Math.random() * subjects.length)];
}

function createSubscriptionPayload(subject = "BlocksSubject", params = {}) {
    return JSON.stringify({
        deliverPolicy: "from_block:0",
        subscribe: [{ subject: subject, params: params }],
    });
}

const API_KEY = "fuel-CPEYX4t94gijC_q2b.U6AXNENm-Uqnhw";
export const WsErrors = new Counter("Errors");

export default function () {
    const url = `wss://stream-mainnet.fuel.network/api/v1/ws?api_key=${API_KEY}`;

    const res = ws.connect(url, {}, (socket) => {
        socket.on("open", () => {
            // Get a random subject but keep params empty
            const randomSubject = getRandomSubject();
            const subPayload = createSubscriptionPayload(randomSubject, {});
            socket.send(subPayload);
        });

        socket.on("message", (data) => {
            const message = JSON.parse(data);

            if (message.error) {
                console.error("WebSocket error:", message.error);
                WsErrors.add(1);
            }

            check(message, {
                "subscription successful": (msg) => msg.response?.subject !== undefined,
            });
        });

        // // biome-ignore lint/suspicious/noExplicitAny: <explanation>
        // socket.on("error", (e: any) => {
        //     console.error("WebSocket error:", e);
        //     console.error("WebSocket error:", e.message);
        //     WsErrors.add(1);
        // });

        socket.setTimeout(() => {
            socket.close();
        }, 30000); // 30s session
    });

    check(res, { "status is 101": (r) => r && r.status === 101 });
    sleep(1);
}
