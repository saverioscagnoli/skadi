import { useState } from "react";

function App() {
  const [count, setCount] = useState(0);

  return (
    <div className="w-screen h-screen ciao">
      <button onClick={() => setCount((c) => c + 1)}>count: {count}</button>
    </div>
  );
}

export default App;
