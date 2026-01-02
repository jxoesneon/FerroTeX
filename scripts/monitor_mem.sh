#!/bin/bash

# FerroTeX Memory Safeguard Monitor
# Usage: ./scripts/monitor_mem.sh <command to run>

if [ $# -lt 1 ]; then
    echo "Usage: $0 <command>"
    exit 1
fi

TARGET_CMD="$@"
THRESHOLD=99

echo "[Safeguard] Starting monitor for: $TARGET_CMD"
echo "[Safeguard] Threshold: $THRESHOLD%"

# Start the target command in the background
$TARGET_CMD &
CMD_PID=$!

# Get system constants
TOTAL_MEM=$(sysctl -n hw.memsize)
PAGE_SIZE=$(vm_stat | grep "page size of" | awk '{print $8}')

function cleanup() {
    if kill -0 $CMD_PID 2>/dev/null; then
        echo "[Safeguard] Cleaning up subprocess $CMD_PID..."
        kill -TERM $CMD_PID 2>/dev/null
        sleep 2
        kill -9 $CMD_PID 2>/dev/null
    fi
}

trap cleanup EXIT

while kill -0 $CMD_PID 2>/dev/null; do
    # Get memory stats from vm_stat
    VM_STATS=$(vm_stat)
    FREE_PAGES=$(echo "$VM_STATS" | grep "Pages free" | awk '{print $3}' | tr -d '.')
    SPECULATIVE_PAGES=$(echo "$VM_STATS" | grep "Pages speculative" | awk '{print $3}' | tr -d '.')
    
    # Calculate free memory
    FREE_MEM=$(( (FREE_PAGES + SPECULATIVE_PAGES) * PAGE_SIZE ))
    USED_MEM=$(( TOTAL_MEM - FREE_MEM ))
    
    # Calculate percentage
    USED_PERCENT=$(( USED_MEM * 100 / TOTAL_MEM ))
    
    if [ $USED_PERCENT -ge $THRESHOLD ]; then
        echo -e "\n\033[0;31m[CRITICAL] Memory usage reached $USED_PERCENT%! Aborting to prevent crash...\033[0m"
        kill -9 $CMD_PID
        exit 1
    fi
    
    # Optional: Print status (don't spam if successful)
    # echo "[Safeguard] Used: $USED_PERCENT%"
    
    sleep 2
done

# Wait for background process to finish and get its exit code
wait $CMD_PID
EXIT_CODE=$?

if [ $EXIT_CODE -eq 0 ]; then
    echo "[Safeguard] Command completed successfully."
else
    echo "[Safeguard] Command failed with exit code $EXIT_CODE."
fi

exit $EXIT_CODE
