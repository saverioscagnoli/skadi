import { useEffect } from "react";

/**
 * Executes a bash command and returns the result as JSON.
 * @param {string} command
 * @param {string[]} [args]
 */
async function exec(command, args) {
  try {
    const response = await fetch("/backend/exec", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        command,
        args: args || undefined,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    return await response.json();
  } catch (error) {
    console.error("Failed to execute command:", error);
    return {
      success: false,
      stdout: "",
      stderr: error.message,
      exit_code: null,
    };
  }
}

/**
 * Custom React hook that starts listening for events based on the provided script.
 * @param {string} script
 * @param {Function} callback
 */
function useListen(script, callback) {
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

    // Set up event listener
    window.addEventListener(script, (e) => callback(e.detail), {
      signal: ctrl.signal,
    });

    return () => {
      ctrl.abort();
    };
  }, []);
}

export { exec, useListen };
