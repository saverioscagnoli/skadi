#!/bin/bash

# Function to get CPU stats from /proc/stat
get_cpu_stats() {
    grep '^cpu ' /proc/stat | awk '{print $2" "$3" "$4" "$5" "$6" "$7" "$8}'
}

# Function to calculate CPU usage percentage from two measurements
calculate_cpu_usage() {
    local prev_stats="$1"
    local curr_stats="$2"
    
    # Parse previous stats
    local prev_user=$(echo $prev_stats | cut -d' ' -f1)
    local prev_nice=$(echo $prev_stats | cut -d' ' -f2)
    local prev_system=$(echo $prev_stats | cut -d' ' -f3)
    local prev_idle=$(echo $prev_stats | cut -d' ' -f4)
    local prev_iowait=$(echo $prev_stats | cut -d' ' -f5)
    local prev_irq=$(echo $prev_stats | cut -d' ' -f6)
    local prev_softirq=$(echo $prev_stats | cut -d' ' -f7)
    
    # Parse current stats
    local curr_user=$(echo $curr_stats | cut -d' ' -f1)
    local curr_nice=$(echo $curr_stats | cut -d' ' -f2)
    local curr_system=$(echo $curr_stats | cut -d' ' -f3)
    local curr_idle=$(echo $curr_stats | cut -d' ' -f4)
    local curr_iowait=$(echo $curr_stats | cut -d' ' -f5)
    local curr_irq=$(echo $curr_stats | cut -d' ' -f6)
    local curr_softirq=$(echo $curr_stats | cut -d' ' -f7)
    
    # Calculate differences
    local user_diff=$((curr_user - prev_user))
    local nice_diff=$((curr_nice - prev_nice))
    local system_diff=$((curr_system - prev_system))
    local idle_diff=$((curr_idle - prev_idle))
    local iowait_diff=$((curr_iowait - prev_iowait))
    local irq_diff=$((curr_irq - prev_irq))
    local softirq_diff=$((curr_softirq - prev_softirq))
    
    # Calculate total time and idle time
    local total_diff=$((user_diff + nice_diff + system_diff + idle_diff + iowait_diff + irq_diff + softirq_diff))
    local idle_total_diff=$((idle_diff + iowait_diff))
    
    # Calculate CPU usage percentage
    if [ $total_diff -gt 0 ]; then
        local usage=$(echo "scale=1; (($total_diff - $idle_total_diff) * 100) / $total_diff" | bc -l)
        echo $usage
    else
        echo "0.0"
    fi
}

# Function to get memory usage percentage
get_mem_usage() {
    free | grep Mem | awk '{printf "%.1f", $3/$2 * 100.0}'
}

# Function to get network stats (bytes per second)
get_network_stats() {
    local interface=$(ip route | grep '^default' | awk '{print $5}' | head -n1)
    if [ -z "$interface" ]; then
        interface="eth0" # fallback
    fi
    local rx_bytes=$(cat /sys/class/net/$interface/statistics/rx_bytes 2>/dev/null || echo "0")
    local tx_bytes=$(cat /sys/class/net/$interface/statistics/tx_bytes 2>/dev/null || echo "0")
    echo "$rx_bytes $tx_bytes"
}

# Function to get disk usage percentage of root filesystem
get_disk_usage() {
    df / | awk 'NR==2 {print $5}' | sed 's/%//'
}

# Check if bc is available for floating point calculations
if ! command -v bc &> /dev/null; then
    echo "Error: 'bc' command is required for CPU calculations. Please install it."
    exit 1
fi

# Initialize previous stats
prev_cpu_stats=$(get_cpu_stats)
prev_net_stats=$(get_network_stats)
prev_rx=$(echo $prev_net_stats | cut -d' ' -f1)
prev_tx=$(echo $prev_net_stats | cut -d' ' -f2)
prev_time=$(date +%s)

sleep 2 # Initial delay to get meaningful stats

while true; do
    # Get current stats
    curr_cpu_stats=$(get_cpu_stats)
    cpu=$(calculate_cpu_usage "$prev_cpu_stats" "$curr_cpu_stats")
    mem=$(get_mem_usage)
    disk=$(get_disk_usage)
    
    # Calculate network throughput
    current_net_stats=$(get_network_stats)
    current_rx=$(echo $current_net_stats | cut -d' ' -f1)
    current_tx=$(echo $current_net_stats | cut -d' ' -f2)
    current_time=$(date +%s)
    
    time_diff=$((current_time - prev_time))
    if [ $time_diff -gt 0 ]; then
        net_down=$(( (current_rx - prev_rx) / time_diff ))
        net_up=$(( (current_tx - prev_tx) / time_diff ))
    else
        net_down=0
        net_up=0
    fi
    
    # Output JSON with timestamp
    printf '{"timestamp": "%s", "cpuUsage": %s, "memUsage": %s, "netUp": %d, "netDown": %d, "disk": %d}\n' \
        "$(date '+%Y-%m-%d %H:%M:%S')" "$cpu" "$mem" "$net_up" "$net_down" "$disk"
    
    # Update previous values
    prev_cpu_stats="$curr_cpu_stats"
    prev_rx=$current_rx
    prev_tx=$current_tx
    prev_time=$current_time
    
    sleep 2
done