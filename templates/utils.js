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
        "Content-Type": "application/json"
      },
      body: JSON.stringify({
        command,
        args: args || undefined
      })
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
      exit_code: null
    };
  }
}

export { exec };