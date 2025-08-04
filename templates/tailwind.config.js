import { homedir } from "os";

export default {
  content: [
    `${homedir()}/.config/skadi/**/*.{html,js,jsx,ts,tsx}`,
    `${homedir()}/.local/share/skadi/**/*.{html,js,jsx,ts,tsx}`,
  ],
  theme: {
    extend: {},
  },
  plugins: [],
};
