import { useEffect, useState } from "react";
import { Props } from "../types";
import { cn } from "../util";
import {
  ArrowDown,
  ArrowUp,
  Cpu,
  MemoryStick,
  Power,
  Server
} from "lucide-react";

type Metrics = {
  cpuUsage: number;
  memUsage: number;
  netUp: number;
  netDown: number;
  disk: number;
};

type NetworkDataPoint = {
  timestamp: number;
  up: number;
  down: number;
};

const formatBytes = (bytes: number): string => {
  if (bytes < 1024) return `${bytes.toFixed(0)}B/s`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}K/s`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}M/s`;
};

const MiniChart: React.FC<{ data: NetworkDataPoint[] }> = ({ data }) => {
  if (data.length < 2) return null;

  const maxValue = Math.max(...data.flatMap(d => [d.up, d.down]));
  const width = 60;
  const height = 20;

  const points = data.map((point, index) => {
    const x = (index / (data.length - 1)) * width;
    const yUp = height - (point.up / maxValue) * height;
    const yDown = height - (point.down / maxValue) * height;
    return { x, yUp, yDown };
  });

  const upPath = points
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p.x} ${p.yUp}`)
    .join(" ");
  const downPath = points
    .map((p, i) => `${i === 0 ? "M" : "L"} ${p.x} ${p.yDown}`)
    .join(" ");

  return (
    <svg width={width} height={height} className="overflow-hidden">
      <path
        d={upPath}
        stroke="#10b981"
        strokeWidth="1"
        fill="none"
        opacity="0.8"
      />
      <path
        d={downPath}
        stroke="#3b82f6"
        strokeWidth="1"
        fill="none"
        opacity="0.8"
      />
    </svg>
  );
};

const SysInfo: React.FC<Props> = ({ exec, useListen }) => {
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [networkHistory, setNetworkHistory] = useState<NetworkDataPoint[]>([]);

  useEffect(() => {
    exec({
      script: "/home/svscagn/.config/skadi/scripts/sysinfo.sh",
      polls: true
    });
  }, []);

  useListen<Metrics>("/home/svscagn/.config/skadi/scripts/sysinfo.sh", data => {
    setMetrics(data);

    setNetworkHistory(prev => {
      const newPoint: NetworkDataPoint = {
        timestamp: Date.now(),
        up: data.netUp,
        down: data.netDown
      };
      const updated = [...prev, newPoint];
      return updated.slice(-30); // Keep only last 30 data points
    });
  });

  return (
    <div
      className={cn(
        "flex items-center gap-2",
        "rounded-lg",
        "bg-black/50 text-white",
        "border-2 border-white/10",
        "px-3 py-1.5",
        "max-w-full overflow-hidden"
      )}
    >
      {metrics && (
        <>
          {/* CPU Usage */}
          <div className={cn("flex items-center gap-1.5 min-w-0")}>
            <Cpu size={14} className="flex-shrink-0" />
            <span
              className="w-12 truncate"
              title={`${metrics.cpuUsage.toFixed(0)}%`}
            >
              {metrics.cpuUsage.toFixed(0)}%
            </span>
          </div>

          {/* Memory Usage */}
          <div className={cn("flex items-center gap-1.5 min-w-0")}>
            <MemoryStick size={14} className="flex-shrink-0" />
            <span
              className="w-12 truncate"
              title={`${metrics.memUsage.toFixed(0)}%`}
            >
              {metrics.memUsage.toFixed(0)}%
            </span>
          </div>

          {/* Network Upload */}
          <div className={cn("flex items-center gap-1.5 min-w-0")}>
            <ArrowUp size={14} className="flex-shrink-0 text-green-400" />
            <span className="w-16 truncate" title={formatBytes(metrics.netUp)}>
              {formatBytes(metrics.netUp)}
            </span>
          </div>

          {/* Network Download */}
          <div className={cn("flex items-center gap-1.5 min-w-0")}>
            <ArrowDown size={14} className="flex-shrink-0 text-blue-400" />
            <span
              className="w-16 truncate"
              title={formatBytes(metrics.netDown)}
            >
              {formatBytes(metrics.netDown)}
            </span>
          </div>

          {/* Network Chart */}
          <div className={cn("flex items-center gap-1.5 min-w-0")}>
            <MiniChart data={networkHistory} />
          </div>

          {/* Disk Usage */}
          <div className={cn("flex items-center gap-1.5 min-w-0")}>
            <Server size={14} className="flex-shrink-0" />
            <span className="w-12  truncate" title={`${metrics.disk}%`}>
              {metrics.disk.toFixed(0)}%
            </span>
          </div>

          {/* Power Button */}
          <button
            onClick={() => exec({ script: "wlogout" })}
            className={cn(
              "flex items-center justify-center",
              "hover:bg-white/10 rounded p-1",
              "transition-colors duration-200",
              "flex-shrink-0"
            )}
            title="Logout"
          >
            <Power size={16} />
          </button>
        </>
      )}
    </div>
  );
};

export { SysInfo };
