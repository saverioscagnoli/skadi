import React, { useEffect } from "react";

// Global callback storage
window.callbacks = new Map();
window.runningProcesses = new Set();

/**
 * @typedef {Object} ExecParams
 * @property {string} script - The script to execute.
 * @property {boolean} [polls] - Whether the script should be polled.
 *
 *
 * @template T
 * @param {ExecParams} params - Parameters for the script execution.
 * @returns {Promise<T>} - A promise that resolves with the result of the script execution.
 */
function exec(
  params = {
    script: "echo 'You need to provide a script to execute'",
    polls: false,
  }
) {
  return new Promise((res, rej) => {
    // Create a process key for tracking
    let processKey = params.script + (params.polls ? "_poll" : "");

    // If this process is already running and it's a polling process, reject
    if (params.polls && window.runningProcesses.has(processKey)) {
      rej("Process is already running");
      return;
    }

    let id = Date.now().toString() + Math.random().toString(36);

    window.callbacks.set(id, {
      resolve: (data) => {
        if (params.polls) window.runningProcesses.delete(processKey);
        res(data);
      },
      reject: (err) => {
        if (params.polls) window.runningProcesses.delete(processKey);
        rej(err);
      },
    });

    // Track polling processes
    if (params.polls) {
      window.runningProcesses.add(processKey);
    }

    let message = {
      id,
      action: "exec",
      ...params,
    };

    window.webkit.messageHandlers.exec.postMessage(JSON.stringify(message));

    // Timeout after 10 seconds
    if (!params.polls) {
      setTimeout(() => {
        if (window.callbacks.has(id)) {
          window.callbacks.delete(id);
          rej(new Error("Backend call timeout"));
        }
      }, 10000);
    }
  });
}

/**
 *
 * @typedef  {Object} Res
 * @property {boolean} success - Indicates if the operation was successful.
 * @property {Object} data - The data returned from the operation.
 * @property {string} [error] - Error message if the operation failed.
 *
 * @param {string} id
 * @param {Res} response
 */
window.callbackHandler = (id, response) => {
  let callback = window.callbacks.get(id);

  if (callback) {
    window.callbacks.delete(id);

    if (response.success) {
      callback.resolve(response.data);
    } else {
      callback.reject(new Error(response.error));
    }
  }
};

/**
 * @template T
 * @param {string} eventName
 * @param {(data: T) => void} handler
 * @param {React.DependencyList} deps
 * @returns {void}
 */
function useListen(event, handler, deps) {
  useEffect(() => {
    let ctrl = new AbortController();

    const wrappedHandler = (e) => {
      try {
        handler(JSON.parse(e.detail));
      } catch (err) {
        console.error("Error in event handler:", err);
      }
    };

    window.addEventListener(event, wrappedHandler, {
      signal: ctrl.signal,
    });

    return () => {
      ctrl.abort();
    };
  }, deps);
}

export { exec, useListen };
