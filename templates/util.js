import  React, { useEffect } from "react";

// Global callback storage
window.callbacks = new Map();

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
function exec(params = { script: "echo 'You need to provide a script to execute'", polls: false, }) {
    return new Promise((res, rej) => {
        let id = Date.now().toString() + Math.random().toString(36);


        window.callbacks.set(id, { resolve: res, reject: rej });

        let message = {
            id,
            action: "exec",
            ...params
        };

        window.webkit.messageHandlers.exec.postMessage(JSON.stringify(message));

        // Timeout after 10 seconds
       if (!params.polls) {
         setTimeout(() => {
            if (window.callbacks.has(id)) {
                window.callbacks.delete(id);
                reject(new Error('Backend call timeout'));
            }
        }, 10000);
       }
    })
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

    window.addEventListener(event, e => handler(JSON.parse(e.detail)), {
      signal: ctrl.signal
    });

    return () => {
      ctrl.abort();
    };
  }, deps);
}

export { exec, useListen };
