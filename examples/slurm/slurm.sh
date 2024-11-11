#!/bin/bash
#SBATCH --ntasks 4
#SBATCH --cpus-per-task 1

# Start one instance of the server with a 5-second fuse and a 4x4 job
addr="$(hostname --ip-address)"
echo -n "Starting server on $(hostname) with IP address $addr... "
srun --ntasks 1 --exclusive -w "$(hostname)" cargo run -q --bin server -- --host "$addr" --port 7878 --fuse 5 4 4 &
echo "Server launched"

echo -n "Launching three client tasks... "
srun --ntasks 3 --exclusive cargo run -q --example slurm -- --host "$addr" --port 7878 --timeout 1 --retries 5 &
echo "Clients launched"
wait
echo "Tasks complete"