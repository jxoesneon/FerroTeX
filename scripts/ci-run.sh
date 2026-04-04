#!/usr/bin/env bash
#
# ci-run.sh - Local CI Testing Wrapper Script for FerroTeX
#
# This script provides a unified way to run CI workflows locally using Docker,
# replicating the GitHub Actions CI environment.
#
# Usage: ./scripts/ci-run.sh [OPTIONS]
#

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DOCKER_IMAGE_NAME="ferrotex-ci-test"
DOCKER_FILE="Dockerfile.ci-test"

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly BOLD='\033[1m'
readonly NC='\033[0m' # No Color

# Timing
START_TIME=""

# Command line flags
BUILD_ONLY=false
TEST_ONLY=false
CHECK_ONLY=false
NO_CACHE=false
VERBOSE=false

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

print_error() {
    echo -e "${RED}[FAIL]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_step() {
    echo -e "${CYAN}[STEP]${NC} ${BOLD}$1${NC}"
}

# Function to print usage information
print_usage() {
    cat <<EOF
${BOLD}FerroTeX Local CI Testing Script${NC}

Runs CI workflows locally using Docker to replicate the GitHub Actions environment.

${BOLD}Usage:${NC}
    ./scripts/ci-run.sh [OPTIONS]

${BOLD}Options:${NC}
    --build-only    Only run cargo build, skip tests
    --test-only     Only run cargo test, skip build (assumes build exists)
    --check-only    Only run cargo check (fastest option)
    --no-cache      Rebuild Docker image from scratch
    --verbose       Show verbose output from cargo commands
    --help          Show this help message

${BOLD}Examples:${NC}
    # Run full CI workflow (check, build, test, clippy)
    ./scripts/ci-run.sh

    # Quick check only (fast feedback)
    ./scripts/ci-run.sh --check-only

    # Build without running tests
    ./scripts/ci-run.sh --build-only

    # Run tests using existing build
    ./scripts/ci-run.sh --test-only

    # Force rebuild Docker image
    ./scripts/ci-run.sh --no-cache

${BOLD}Notes:${NC}
    - Requires Docker to be installed and running
    - First run will build the CI Docker image (may take several minutes)
    - Build artifacts are cached in ./target/
    - The script respects Ctrl+C for graceful interruption

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --build-only)
                BUILD_ONLY=true
                shift
                ;;
            --test-only)
                TEST_ONLY=true
                shift
                ;;
            --check-only)
                CHECK_ONLY=true
                shift
                ;;
            --no-cache)
                NO_CACHE=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                print_usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                print_usage
                exit 1
                ;;
        esac
    done

    # Validate mutually exclusive options
    local exclusive_count=0
    $BUILD_ONLY && ((exclusive_count++))
    $TEST_ONLY && ((exclusive_count++))
    $CHECK_ONLY && ((exclusive_count++))

    if [[ $exclusive_count -gt 1 ]]; then
        print_error "Options --build-only, --test-only, and --check-only are mutually exclusive"
        exit 1
    fi
}

# Function to check if Docker is installed and running
check_docker() {
    print_step "Checking Docker installation"

    if ! command -v docker &>/dev/null; then
        print_error "Docker is not installed"
        echo ""
        echo "Please install Docker:"
        echo "  - macOS: https://docs.docker.com/desktop/install/mac-install/"
        echo "  - Linux: https://docs.docker.com/engine/install/"
        echo "  - Windows: https://docs.docker.com/desktop/install/windows-install/"
        exit 1
    fi

    if ! docker info &>/dev/null; then
        print_error "Docker daemon is not running"
        echo ""
        echo "Please start Docker Desktop or the Docker service"
        exit 1
    fi

    print_success "Docker is installed and running"
}

# Function to setup signal handlers for graceful interruption
setup_signal_handlers() {
    cleanup() {
        echo ""
        print_warning "Interrupted by user"
        exit 130
    }
    trap cleanup INT TERM
}

# Function to start timing
start_timer() {
    START_TIME=$(date +%s)
}

# Function to get elapsed time
get_elapsed() {
    local end_time=$(date +%s)
    local elapsed=$((end_time - START_TIME))
    printf "%02d:%02d" $((elapsed / 60)) $((elapsed % 60))
}

# Function to print timing report
print_timing() {
    local elapsed=$(get_elapsed)
    echo -e "${CYAN}[TIME]${NC} Elapsed: ${BOLD}${elapsed}${NC}"
}

# Function to build Docker image
build_docker_image() {
    print_step "Building Docker CI image"

    if [[ ! -f "${PROJECT_ROOT}/${DOCKER_FILE}" ]]; then
        print_error "Dockerfile not found: ${DOCKER_FILE}"
        echo ""
        echo "Please ensure ${DOCKER_FILE} exists in the project root."
        exit 1
    fi

    local build_args=()
    if $NO_CACHE; then
        build_args+=("--no-cache")
        print_info "Building without cache (--no-cache specified)"
    fi

    start_timer

    if $VERBOSE; then
        docker build ${build_args[@]+"${build_args[@]}"} -f "${PROJECT_ROOT}/${DOCKER_FILE}" -t "${DOCKER_IMAGE_NAME}" "${PROJECT_ROOT}"
    else
        docker build ${build_args[@]+"${build_args[@]}"} -f "${PROJECT_ROOT}/${DOCKER_FILE}" -t "${DOCKER_IMAGE_NAME}" "${PROJECT_ROOT}" 2>&1 | \
            while read -r line; do
                [[ "$line" =~ ^(Step| --->|Successfully) ]] && echo "$line"
            done
    fi

    print_success "Docker image built successfully"
    print_timing
}

# Function to check if Docker image exists
docker_image_exists() {
    docker images -q "${DOCKER_IMAGE_NAME}" 2>/dev/null | grep -q .
}

# Function to run a command in Docker
run_docker_command() {
    local cmd="$1"
    local description="$2"

    print_step "$description"
    start_timer

    local docker_args=(
        "run"
        "--rm"
        "-v" "${PROJECT_ROOT}:/workspace"
        "-w" "/workspace"
        "-e" "CARGO_HOME=/workspace/.cargo-docker"
    )

    # Add TTY if interactive
    if [[ -t 0 ]]; then
        docker_args+=("-t")
    fi

    docker_args+=("${DOCKER_IMAGE_NAME}")

    if $VERBOSE; then
        docker "${docker_args[@]}" bash -c "$cmd"
    else
        # Show spinner while running
        docker "${docker_args[@]}" bash -c "$cmd" &
        local pid=$!

        local spin_chars='⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏'
        local i=0

        while kill -0 $pid 2>/dev/null; do
            printf "\r%s Running... " "${spin_chars:$i:1}"
            i=$(( (i + 1) % 10 ))
            sleep 0.1
        done

        wait $pid
        local exit_code=$?
        printf "\r\033[K"

        if [[ $exit_code -ne 0 ]]; then
            return $exit_code
        fi
    fi

    print_success "$description completed"
    print_timing
}

# Function to run cargo check
run_check() {
    run_docker_command "cargo check --workspace" "Running cargo check"
}

# Function to run cargo build
run_build() {
    local build_flags="--workspace"
    $VERBOSE && build_flags="${build_flags} --verbose"
    run_docker_command "cargo build ${build_flags}" "Running cargo build"
}

# Function to run cargo test
run_test() {
    local test_cmd="cargo nextest run --workspace"
    $VERBOSE && test_cmd="${test_cmd} --verbose"

    # Run nextest
    run_docker_command "$test_cmd" "Running cargo nextest"

    # Run doctests
    local doc_test_cmd="cargo test --doc --workspace"
    $VERBOSE && doc_test_cmd="${doc_test_cmd} --verbose"
    run_docker_command "$doc_test_cmd" "Running cargo doctests"
}

# Function to run cargo clippy
run_clippy() {
    run_docker_command "cargo clippy --workspace -- -D warnings" "Running cargo clippy"
}

# Main execution
main() {
    parse_args "$@"
    setup_signal_handlers

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}  FerroTeX Local CI Testing${NC}"
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
    echo ""

    # Check Docker
    check_docker

    # Build image if needed
    if $NO_CACHE || ! docker_image_exists; then
        build_docker_image
    else
        print_info "Using existing Docker image: ${DOCKER_IMAGE_NAME}"
    fi

    echo ""
    echo -e "${BOLD}───────────────────────────────────────────────────────${NC}"
    echo ""

    # Track overall success
    local overall_start_time=$(date +%s)
    local failed=false

    # Execute based on mode
    if $CHECK_ONLY; then
        if ! run_check; then
            failed=true
        fi
    elif $BUILD_ONLY; then
        if ! run_build; then
            failed=true
        fi
    elif $TEST_ONLY; then
        if ! run_test; then
            failed=true
        fi
    else
        # Full CI workflow

        # Step 1: Check
        if ! run_check; then
            failed=true
        fi

        # Step 2: Build
        if ! $failed && ! run_build; then
            failed=true
        fi

        # Step 3: Test
        if ! $failed && ! run_test; then
            failed=true
        fi

        # Step 4: Clippy (optional, don't fail on it)
        if ! $failed; then
            run_clippy || print_warning "Clippy found warnings (non-fatal)"
        fi
    fi

    # Calculate total time
    local overall_end_time=$(date +%s)
    local total_elapsed=$((overall_end_time - overall_start_time))
    local total_formatted=$(printf "%02d:%02d" $((total_elapsed / 60)) $((total_elapsed % 60)))

    echo ""
    echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"

    if $failed; then
        echo -e "${RED}${BOLD}  CI FAILED${NC}"
        echo -e "${BOLD}  Total time: ${total_formatted}${NC}"
        echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
        echo ""
        exit 1
    else
        echo -e "${GREEN}${BOLD}  CI PASSED${NC}"
        echo -e "${BOLD}  Total time: ${total_formatted}${NC}"
        echo -e "${BOLD}═══════════════════════════════════════════════════════${NC}"
        echo ""
        exit 0
    fi
}

# Run main function
main "$@"
