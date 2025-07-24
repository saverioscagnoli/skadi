// import type React from "react";
// import { useEffect } from "react";

// type ExecOptions = { polls: boolean };
// type ExecReturn = <T = any>(script: string, options: ExecOptions) => Promise<T>;

// function exec(label: string): ExecReturn {
//   return async <T = any>(
//     script: string,
//     options: ExecOptions = { polls: false }
//   ): Promise<T> => {
//     if (!options) {
//       options = { polls: false };
//     }

//     if (options.polls === undefined) {
//       options.polls = false;
//     }

//     let res = await fetch("http://localhost:3499/exec", {
//       method: "POST",
//       headers: { "Content-Type": "application/json", "x-window-label": label },
//       body: JSON.stringify({ command: script, polls: options.polls })
//     });

//     if (!res.ok) {
//       let errorText = await res.text();
//       throw new Error(`Failed to execute script: ${errorText}`);
//     }

//     let data = await res.json();
//     return data as T;
//   };
// }

// type CustomEvent<T = any> = { detail: T };

// function useListen<T = any>(
//   event: string,
//   handler: (payload: T) => any,
//   deps: React.DependencyList
// ) {
//   useEffect(() => {
//     let ctrl = new AbortController();

//     // @ts-ignore
//     window.addEventListener(event, (e: CustomEvent<T>) => handler(e.detail), {
//       signal: ctrl.signal
//     });

//     return () => {
//       ctrl.abort();
//     };
//   }, deps);
// }

// export { exec, useListen };

class IPCSocket {
  private ws: WebSocket | null = null;
  private messageHandlers = new Map<string, (data: any) => void>();

  connect(port: number) {
    this.ws = new WebSocket(`ws://localhost:${port}/ws`);

    this.ws.onmessage = event => {
      const data = JSON.parse(event.data);
      const handler = this.messageHandlers.get(data.type);
      if (handler) {
        handler(data);
      }
    };
  }

  send(type: string, data: any): Promise<any> {
    return new Promise((resolve, reject) => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        reject(new Error("WebSocket not connected"));
        return;
      }

      const id = Math.random().toString(36);
      const message = { id, type, ...data };

      // Set up response handler
      const timeout = setTimeout(() => {
        this.messageHandlers.delete(id);
        reject(new Error("Request timeout"));
      }, 5000);

      this.messageHandlers.set(id, response => {
        clearTimeout(timeout);
        this.messageHandlers.delete(id);
        resolve(response);
      });

      this.ws.send(JSON.stringify(message));
    });
  }
}

const ipcSocket = new IPCSocket();
ipcSocket.connect(3499);

export const exec =
  (label: string) => async (path: string, args?: string[]) => {
    return ipcSocket.send("exec", { label, path, args });
  };

export const useListen = (event: string, handler: (data: any) => void) => {};
