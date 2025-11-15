import { useEffect } from "react";

type BackendMessage = {
  success: boolean;
  stdout: string;
  stderr: string;
};

async function exec(
  command: string,
  args: string[] = [],
): Promise<BackendMessage> {
  try {
    let response = await fetch("/backend/exec", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        command,
        args,
      }),
    });

    return await response.json();
  } catch (err) {
    console.error("Failed to contact backend:", err.message);

    return {
      success: false,
      stdout: "",
      stderr: "",
    };
  }
}

function useListen<T>(
  script: string,
  args: string[],
  callback: (payload: T) => void,
) {
  useEffect(() => {
    let ctrl = new AbortController();

    // Start listening on the backend
    fetch("/backend/listen", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        script,
        args,
        // Get the widget label from the document
        // (See common/src/)
        widget_label: document.title,
      }),
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
      })
      .catch((error) => {
        console.error("Failed to start listening:", error);
      });

    // Set up event listener with unique event name that includes args
    let eventName = args.length > 0 ? `${script} ${args.join(" ")}` : script;
    console.log(eventName);
    // @ts-ignore
    window.addEventListener(
      eventName,
      (e: CustomEvent<T>) => callback(e.detail),
      {
        signal: ctrl.signal,
      },
    );

    return () => {
      ctrl.abort();
    };
  }, []);
}

const windowHandle = {
  show: async () => {
    try {
      let response = await fetch("/backend/window/show", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          label: document.title,
        }),
      });

      if (!response.ok) {
        console.error(`Failed to show window: ${response.status}`);
      }
    } catch (err) {
      console.error("Failed to show window", err.message);
    }
  },
  hide: async () => {
    try {
      let response = await fetch("/backend/window/hide", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          label: document.title,
        }),
      });

      if (!response.ok) {
        console.error(`Failed to hide window: ${response.status}`);
      }
    } catch (err) {
      console.error("Failed to hide window", err.message);
    }
  },
};

export { exec, useListen, windowHandle as window };
