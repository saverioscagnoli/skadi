import { useEffect } from "react";

type CommandOutput = {
  success: boolean;
  stdout: string;
  stderr: string;
};

// For single exec request, don't use websockets
// just for simplcity, use post requests.
// They're more than enough for this use case

async function exec(command: string, args?: string[]): Promise<CommandOutput> {
  try {
    let response = await fetch("/exec", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        command,
        args: args || [],
        widgetLabel: document.title,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.json();
  } catch (err) {
    console.error(
      "Request failed. The return output is the http error, not the command stderr.",
    );
    return {
      success: false,
      stdout: "",
      stderr: err.message,
    };
  }
}

// For streaming, use websockets.
// This may be overkill, but using js evaluation in the backend
// for dispatching events kept causing problems and generally not
// suitable for faster streams.

let websocket: WebSocket | null = null;
let websocketRefCount = 0;
let streamIdCounter = 0;

// Map of stream ID to callback
const streamCallbacks = new Map<string, (line: string) => void>();

// Websocket singleton
function getWebsocket(): WebSocket {
  if (!websocket || websocket.readyState === WebSocket.CLOSED) {
    websocket = new WebSocket("ws://localhost:10978/ws");

    websocket.addEventListener("open", () => {
      websocket!.send(`IDENTIFY ${document.title}`);
    });

    websocket.addEventListener("message", (event) => {
      try {
        const message = JSON.parse(event.data);

        if (message.streamId && streamCallbacks.has(message.streamId)) {
          const callback = streamCallbacks.get(message.streamId)!;
          callback(message.data);
        }
      } catch (err) {
        console.error("Failed to parse WebSocket message:", err);
      }
    });

    websocket.addEventListener("error", (error) => {
      console.error("WebSocket error:", error);
    });

    websocket.addEventListener("close", () => {
      console.log("WebSocket connection closed");
    });
  }

  websocketRefCount++;
  return websocket;
}

function releaseWebsocket() {
  websocketRefCount--;
  if (websocketRefCount === 0 && websocket) {
    websocket.close();
    websocket = null;
  }
}

function useListen(
  command: string,
  args: string[],
  callback: (line: string) => void,
) {
  useEffect(() => {
    const streamId = `stream_${++streamIdCounter}_${Date.now()}`;

    streamCallbacks.set(streamId, callback);

    const ws = getWebsocket();

    const startStream = () => {
      ws.send(`LISTEN ${streamId} ${command} ${args?.join(" ") || ""}`);
    };

    if (ws.readyState === WebSocket.OPEN) {
      startStream();
    } else {
      ws.addEventListener("open", startStream, { once: true });
    }

    return () => {
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(`STOP_STREAM ${streamId}`);
      }

      streamCallbacks.delete(streamId);
      releaseWebsocket();
    };
  }, []);
}

export { exec, useListen };
