#!/bin/sh
# SPDX-License-Identified: BSD-3-Clause
#
# TR-101 Machine Report - Dynamic Version
# Copyright © 2024, U.S. Graphics, LLC. BSD-3-Clause License.
# Copyright © 2025, Dmitry Achkasov <achkasov.dmitry@live.com>.

# Global configuration
MIN_NAME_LEN=5
MAX_NAME_LEN=10
MIN_DATA_LEN=5
MAX_DATA_LEN=32
BORDERS_AND_PADDING=7
REPORT_TITLE="RETRO-OS PHASE 0"
APP_NAME="TR-101 MACHINE REPORT"

# Mode flags
USE_MOCK_DATA=${USE_MOCK_DATA:-0}
CONTINUOUS_MODE=${CONTINUOUS_MODE:-0}
REFRESH_RATE=${REFRESH_RATE:-1}

# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

debug() {
    # Uncomment for debug logging
    # printf "DEBUG: $@\n" >&2
    printf ""
}

max_length() {
    max_len=$MIN_DATA_LEN
    len=0

    for str in "$@"; do
        len=${#str}
        if [ "$len" -gt "$max_len" ]; then
            max_len=$len
        fi
    done

    if [ "$max_len" -lt "$MAX_DATA_LEN" ]; then
        printf '%s' "$max_len"
    else
        printf '%s' "$MAX_DATA_LEN"
    fi
}

bar_graph() {
    percent=0
    num_blocks=0
    width=$1
    graph=""
    used=$2
    total=$3

    # Use awk for float comparison
    is_zero=$(awk -v total="$total" 'BEGIN { print (total == 0 || total == "") ? 1 : 0 }')
    if [ "$is_zero" -eq 1 ]; then
        percent=0
    else
        percent=$(awk -v used="$used" -v total="$total" 'BEGIN { printf "%.2f", (used / total) * 100 }')
    fi

    num_blocks=$(awk -v percent="$percent" -v width="$width" 'BEGIN { printf "%d", (percent / 100) * width }')

    i=0
    while [ "$i" -lt "$num_blocks" ]; do
        graph="${graph}█"
        i=$(( i + 1 ))
    done

    while [ "$i" -lt "$width" ]; do
         graph="${graph}░"
        i=$(( i + 1 ))
    done

    printf "%s" "${graph}"
}

# ============================================================================
# DATA COLLECTION FUNCTIONS
# ============================================================================

collect_os_info() {
    if [ "$USE_MOCK_DATA" -eq 1 ]; then
        printf "macOS\nDarwin 25.1.0 "
        return
    fi

    os_name="???"
    if [ -f /etc/os-release ]; then
        NAME="$( grep "^NAME=" /etc/os-release | cut -d'=' -f2 | tr -d '"')"
        VERSION="$( grep \"^VERSION=\" /etc/os-release | cut -d'=' -f2 )"
        VERSION_CODENAME="$( grep \"^VERSION_CODENAME=\" /etc/os-release | cut -d'=' -f2 )"
        os_name="${NAME} ${VERSION} ${VERSION_CODENAME}"
    else
        os_name="$(uname -s)"
        case $os_name in
            Darwin)
            os_name=$(sw_vers | grep "ProductName:" | tr -d "\t" | cut -d ":" -f 2)
            ;;
        esac
    fi

    os_kernel=$({ uname; uname -r; } | tr '\n' ' ')

    printf "%s\n%s" "$os_name" "$os_kernel"
}

collect_network_info() {
    if [ "$USE_MOCK_DATA" -eq 1 ]; then
        printf "hostname.local\n192.168.1.100\nNot connected\n192.168.1.1 8.8.8.8\nbwl"
        return
    fi

    net_current_user=$(whoami)

    if ! [ "$(command -v hostname)" ]; then
        net_hostname=$(grep -w "$(uname -n)" /etc/hosts | awk '{print $2}' | head -n 1)
    else
        net_hostname=$(hostname)
    fi
    if [ -z "$net_hostname" ]; then net_hostname="Not Defined"; fi

    # Get IP address
    ipv4_address=""
    ipv6_address=""
    if command -v ifconfig >/dev/null 2>&1; then
        ipv4_address=$(ifconfig | awk '
            /^[a-z]/ {iface=$1}
            iface != "lo:" && iface !="lo0:" && iface !~ /^docker/ && /inet / && !found_ipv4 {found_ipv4=1; print $2}')
        if [ -z "$ipv4_address" ]; then
            ipv6_address=$(ifconfig | awk '
                /^[a-z]/ {iface=$1}
                iface != "lo:" && iface != "lo0:" && iface !~ /^docker/ && /inet6 / && !found_ipv6 {found_ipv6=1; print $2}')
        fi
    elif command -v ip >/dev/null 2>&1; then
        ipv4_address=$(ip -o -4 addr show | awk '
            $2 != "lo" && $2 !~ /^docker/ {split($4, a, "/"); if (!found_ipv4++) print a[1]}')
        if [ -z "$ipv4_address" ]; then
            ipv6_address=$(ip -o -6 addr show | awk '
                $2 != "lo" && $2 !~ /^docker/ {split($4, a, "/"); if (!found_ipv6++) print a[1]}')
        fi
    fi

    if [ -z "$ipv4_address" ] && [ -z "$ipv6_address" ]; then
        net_machine_ip="No IP found"
    else
        net_machine_ip="${ipv4_address:-$ipv6_address}"
    fi

    net_client_ip=$(who am i | awk '{print $NF}')
    if [ -z "$net_client_ip" ] ; then
        net_client_ip="Not connected"
    fi
    case "$net_client_ip" in
        "("*) ;;
        *) net_client_ip="Not connected" ;;
    esac
    net_client_ip=$(echo "$net_client_ip" | tr -d '()')

    net_dns_ip=$(grep '^nameserver [0-9.]' /etc/resolv.conf | cut -d' ' -f2 | tr '\n' ' ')

    printf "%s\n%s\n%s\n%s\n%s" "$net_hostname" "$net_machine_ip" "$net_client_ip" "$net_dns_ip" "$net_current_user"
}

collect_cpu_info() {
    if [ "$USE_MOCK_DATA" -eq 1 ]; then
        printf "Apple M3\n8\n8\nBare Metal\nN/A\n2.5\n2.8\n3.1\n8"
        return
    fi

    case "$(uname)" in
        Darwin)
            cpu_model="$(sysctl -n machdep.cpu.brand_string)"
            cpu_cores_per_socket="$(sysctl -n machdep.cpu.core_count)"
            cpu_sockets="$(sysctl -n hw.physicalcpu)"
            cpu_cores="$(sysctl -n hw.ncpu)"
            ;;
        SunOS)
            cpu_model="$(kstat -C -m cpu_info -i 0 -s brand | cut -f 5 -d ':')"
            cpu_cores_per_socket="$(psrinfo -tc)"
            cpu_sockets="$(psrinfo -p)"
            if [ "$(smbios -t SMB_TYPE_SYSTEM | grep "Product" | tr -d ' ' |  cut -d ':' -f2)" = "VirtualMachine" ]; then
                cpu_hypervisor=$(smbios -t SMB_TYPE_BIOS | grep "Version String" | cut -d ':' -f2 | sed 's/^[[:space:]]* //')
            fi
            cpu_cores="$(nproc --all)"
            ;;
        *)
            if ! command -v lscpu >/dev/null 2>&1; then
                printf "ERROR: \`lscpu\` utility is not found" >&2
                exit 1
            fi
            if ! command -v nproc >/dev/null 2>&1; then
                printf "ERROR: \`nproc\` utility is not found" >&2
                exit 1
            fi
            cpu_cores_per_socket="$(lscpu | grep 'Core(s) per socket' | cut -f 2 -d ':'| awk '{$1=$1}1')"
            cpu_model="$(lscpu | grep 'Model name' | grep -v 'BIOS' | cut -f 2 -d ':' | awk '{print $1 " "  $2 " " $3 " " $4}')"
            cpu_sockets="$(lscpu | grep 'Socket(s)' | cut -f 2 -d ':' | awk '{$1=$1}1')"
            cpu_hypervisor="$(lscpu | grep 'Hypervisor vendor' | cut -f 2 -d ':' | awk '{$1=$1}1')"
            cpu_cores="$(nproc --all)"
            ;;
    esac
    if [ -z "$cpu_hypervisor" ]; then
        cpu_hypervisor="Bare Metal"
    fi

    # CPU frequency
    case "$(uname)" in
      Linux)
        cpu_freq="$(grep 'cpu MHz' /proc/cpuinfo | cut -f 2 -d ':' | awk 'NR==1 { printf "%.2f", $1 / 1000 }')"
        ;;
      Darwin)
         cpu_freq="$(sysctl -n hw.cpufrequency 2>/dev/null | awk 'NR==1 { printf "%.2f", $1 / 1000000000 }')"
         if [ -z "$cpu_freq" ]; then
             cpu_freq="$(sysctl -n hw.cpufrequency_max 2>/dev/null | awk 'NR==1 { printf "%.2f", $1 / 1000000000 }')"
         fi
         if [ -z "$cpu_freq" ] || [ "$cpu_freq" = "0.00" ]; then
             cpu_freq="N/A"
         fi
         ;;
      FreeBSD)
        cpu_freq="$(sysctl -n dev.cpu.0.freq | awk 'NR==1 { printf "%.2f", $1 / 1000 }')"
          ;;
      SunOS)
         cpu_freq="$(kstat -C -m cpu_info -i 0 -s clock_MHz | cut -f 5 -d ':' | awk 'NR==1 { printf "%.2f", $1 / 1000 }')"
        ;;
      *)
          cpu_freq="???"
          ;;
    esac

    # Load averages
    case "$(uname)" in
        FreeBSD|Darwin)
            load_avg_1min=$(uptime | awk -F'load averages: ' '{print $2}' | cut -d ',' -f1 | tr -d ' ')
            load_avg_5min=$(uptime | awk -F'load averages: ' '{print $2}' | cut -d ',' -f2 | tr -d ' ')
            load_avg_15min=$(uptime| awk -F'load averages: ' '{print $2}' | cut -d ',' -f3 | tr -d ' ')
            ;;
        Linux|SunOS|*)
            load_avg_1min=$(uptime | awk -F'load average: ' '{print $2}' | cut -d ',' -f1 | tr -d ' ')
            load_avg_5min=$(uptime | awk -F'load average: ' '{print $2}' | cut -d ',' -f2 | tr -d ' ')
            load_avg_15min=$(uptime| awk -F'load average: ' '{print $2}' | cut -d ',' -f3 | tr -d ' ')
            ;;
    esac

    printf "%s\n%s\n%s\n%s\n%s\n%s\n%s\n%s\n%s" "$cpu_model" "$cpu_cores_per_socket" "$cpu_sockets" "$cpu_hypervisor" "$cpu_freq" "$load_avg_1min" "$load_avg_5min" "$load_avg_15min" "$cpu_cores"
}

collect_memory_info() {
    if [ "$USE_MOCK_DATA" -eq 1 ]; then
        printf "16384000\n4096000\n12288000\n75.00\n16.00\n12.00"
        return
    fi

    case "$(uname)" in
      Linux)
        mem_total=$(grep 'MemTotal' /proc/meminfo | awk '{print $2}')
        mem_available=$(grep 'MemAvailable' /proc/meminfo | awk '{print $2}')
        ;;
      FreeBSD)
        mem_total=$(( $(sysctl -n hw.physmem) / 1024 ))
        mem_available=$(( ($(sysctl -n vm.stats.vm.v_free_count) + $(sysctl -n vm.stats.vm.v_inactive_count)) * $(sysctl -n hw.pagesize) / 1024 ))
        ;;
      Darwin)
        mem_total=$(( $(sysctl -n hw.physmem) / 1024 ))
        mem_available=$(( $(sysctl -n hw.pagesize) * $(vm_stat | grep "Pages free:" | tr -d " ." | cut -d ":" -f 2) / 1024 ))
        ;;
      SunOS)
        mem_total=$(( $(kstat -C -m unix -n system_pages -s physmem | cut -d':' -f5) * 4 ))
        mem_available=$(( $(kstat -C -m unix -n system_pages -s freemem | cut -d':' -f5) * 4 ))
        ;;
      *)
        mem_total="???"
        mem_available="???"
        ;;
    esac

    mem_used=$((mem_total - mem_available))
    mem_percent=$(awk -v used="$mem_used" -v total="$mem_total" 'BEGIN { printf "%.2f", (used / total) * 100 }')
    mem_total_gb=$(echo "$mem_total" | awk '{ printf "%.2f", $1 / (1024 * 1024) }')
    mem_used_gb=$(echo "$mem_used" | awk '{ printf "%.2f", $1 / (1024 * 1024) }')

    printf "%s\n%s\n%s\n%s\n%s\n%s" "$mem_total" "$mem_available" "$mem_used" "$mem_percent" "$mem_total_gb" "$mem_used_gb"
}

collect_disk_info() {
    if [ "$USE_MOCK_DATA" -eq 1 ]; then
        printf "0\n500.00\n50.00\n10.00\nHEALTH O.K."
        return
    fi

    zfs_present=0
    if command -v zpool >/dev/null 2>&1; then
        zfs_filesystem=$( zpool list -H -o name | tail -n 1 )
    fi

    if command -v zfs >/dev/null 2>&1 && [ "$(( $(zpool list -H 2>/dev/null | wc -l) ))" -gt 0 ]; then
        zfs_present=1
        zfs_health=$(zpool status -x "$zfs_filesystem" | grep -q "is healthy" && echo  "HEALTH O.K.")
        zfs_available=$(zfs get -o value -Hp available "$zfs_filesystem")
        zfs_used=$(zfs get -o value -Hp used "$zfs_filesystem")
        zfs_available_gb=$(echo "$zfs_available" | awk '{ printf "%.2f", $1 / (1024 * 1024 * 1024) }')
        zfs_used_gb=$(echo "$zfs_used" | awk '{ printf "%.2f", $1 / (1024 * 1024 * 1024) }')
        disk_percent=$(awk -v used="$zfs_used" -v available="$zfs_available" 'BEGIN { printf "%.2f", (used / available) * 100 }')

        printf "%s\n%s\n%s\n%s\n%s" "$zfs_present" "$zfs_available_gb" "$zfs_used_gb" "$disk_percent" "$zfs_health"
    else
        root_partition="/"
        root_used=$(df -m "$root_partition" | awk 'NR==2 {print $3}')
        root_total=$(df -m "$root_partition" | awk 'NR==2 {print $2}')
        root_total_gb=$(awk -v total="$root_total" 'BEGIN { printf "%.2f", total / 1024 }')
        root_used_gb=$(awk -v used="$root_used" 'BEGIN { printf "%.2f", used / 1024 }')
        disk_percent=$(awk -v used="$root_used" -v total="$root_total" 'BEGIN { printf "%.2f", (used / total) * 100 }')

        printf "%s\n%s\n%s\n%s\n%s" "$zfs_present" "$root_total_gb" "$root_used_gb" "$disk_percent" ""
    fi
}

collect_login_info() {
    if [ "$USE_MOCK_DATA" -eq 1 ]; then
        printf "Oct 20 01:38\n0\n\n2d 5h"
        return
    fi

    last_login_time="Never logged in"
    last_login_ip_present=0
    last_login_ip=""

    if command -v lastlog >/dev/null 2>&1; then
        last_login=$(lastlog -u "$USER")
        last_login_ip=$(echo "$last_login" | awk 'NR==2 {print $3}')
        last_login_ip=$( echo "$last_login_ip" | sed -n '/^[0-9]\{1,3\}\(\.[0-9]\{1,3\}\)\{3\}$/p' )

        if [ -z "$last_login_ip" ]; then
            last_login_ip_present=0
        else
            if echo "$last_login_ip" | awk -F. 'NF==4 && $1<=255 && $2<=255 && $3<=255 && $4<=255' >/dev/null 2>&1; then
                last_login_ip_present=1
                last_login_time=$(echo "$last_login" | awk 'NR==2 {print $6, $7, $10, $8}')
            else
                last_login_time=$(echo "$last_login" | awk 'NR==2 {print $4, $5, $8, $6}')
            fi
        fi
    else
        case $(uname) in
            FreeBSD)
                last_login_ip=$(lastlogin --libxo json,pretty "$USER" | awk -F'"' '/"from"/ {print $4}')
                if [ -n "$last_login_ip" ]; then
                    last_login_ip_present=1
                fi
                last_login_time=$(lastlogin --libxo json,pretty "$USER" | awk -F '"' '/"login-time"/ {print $4}')
                ;;
            Darwin)
                last_login=$(last -1 "$USER" | head -n 1)
                if echo "$last_login" | grep -q "^$USER"; then
                    last_login_ip=$(echo "$last_login" | awk '{print $3}')
                    if echo "$last_login_ip" | awk -F. 'NF==4 && $1<=255 && $2<=255 && $3<=255 && $4<=255' >/dev/null 2>&1; then
                        last_login_ip_present=1
                        last_login_time=$(echo "$last_login" | awk '{print $4, $5, $6, $7}')
                    else
                        last_login_time=$(echo "$last_login" | awk '{print $4, $5, $6}')
                    fi
                else
                    last_login_time="Never logged in"
                fi
                ;;
            *)
                last_login=$(last "$USER" | head -n 1)
                last_login_ip=$(echo "$last_login" | awk 'NR==1 {print $3}')
                if echo "$last_login_ip" | awk -F. 'NF==4 && $1<=255 && $2<=255 && $3<=255 && $4<=255' >/dev/null 2>&1; then
                    last_login_ip_present=1
                    last_login_time=$(echo "$last_login" | awk 'NR==1 {print $4, $5, $6, $7}')
                else
                    last_login_time=$(echo "$last_login" | awk 'NR==1 {print $3, $4, $5, $6}')
                fi
                ;;
        esac
    fi

    sys_uptime=$(uptime | cut -d',' -f1 \
                        | sed 's/^[^ ]* //' \
                        | sed 's/^[^ ]* //' \
                        | sed 's/^[ ]* //' \
                        | sed "s/up[ ][[:space:]]*//" \
                        | sed 's/[[:space:]]*day\(s*\)/d/' \
                        | sed 's/[[:space:]]*hour\(s*\)/h/' \
                        | sed 's/[[:space:]]*minute\(s*\)/m/')

    printf "%s\n%s\n%s\n%s" "$last_login_time" "$last_login_ip_present" "$last_login_ip" "$sys_uptime"
}

# ============================================================================
# RENDERING FUNCTIONS
# ============================================================================

render_ui() {
    # Parse all arguments
    os_name="$1"
    os_kernel="$2"
    net_hostname="$3"
    net_machine_ip="$4"
    net_client_ip="$5"
    net_dns_ip="$6"
    net_user="$7"
    cpu_model="$8"
    cpu_cores_per_socket="$9"
    shift 9
    cpu_sockets="$1"
    cpu_hypervisor="$2"
    cpu_freq="$3"
    load_avg_1min="$4"
    load_avg_5min="$5"
    load_avg_15min="$6"
    cpu_cores="$7"
    mem_total="$8"
    mem_available="$9"
    shift 9
    mem_used="$1"
    mem_percent="$2"
    mem_total_gb="$3"
    mem_used_gb="$4"
    zfs_present="$5"
    disk_total_gb="$6"
    disk_used_gb="$7"
    disk_percent="$8"
    disk_health="$9"
    shift 9
    last_login_time="$1"
    last_login_ip_present="$2"
    last_login_ip="$3"
    sys_uptime="$4"

    # Calculate bar graph width (fixed at 32 chars for now)
    BAR_WIDTH=32

    # Generate bar graphs
    cpu_1min_bar=$(bar_graph "$BAR_WIDTH" "$load_avg_1min" "$cpu_cores")
    cpu_5min_bar=$(bar_graph "$BAR_WIDTH" "$load_avg_5min" "$cpu_cores")
    cpu_15min_bar=$(bar_graph "$BAR_WIDTH" "$load_avg_15min" "$cpu_cores")
    mem_bar=$(bar_graph "$BAR_WIDTH" "$mem_used" "$mem_total")

    if [ "$zfs_present" -eq 1 ]; then
        # ZFS uses used/available ratio
        disk_used_bytes=$(awk -v gb="$disk_used_gb" 'BEGIN { printf "%.0f", gb * 1024 * 1024 * 1024 }')
        disk_total_bytes=$(awk -v gb="$disk_total_gb" 'BEGIN { printf "%.0f", gb * 1024 * 1024 * 1024 }')
        disk_bar=$(bar_graph "$BAR_WIDTH" "$disk_used_bytes" "$disk_total_bytes")
    else
        disk_used_mb=$(awk -v gb="$disk_used_gb" 'BEGIN { printf "%.0f", gb * 1024 }')
        disk_total_mb=$(awk -v gb="$disk_total_gb" 'BEGIN { printf "%.0f", gb * 1024 }')
        disk_bar=$(bar_graph "$BAR_WIDTH" "$disk_used_mb" "$disk_total_mb")
    fi

    # Calculate CURRENT_LEN for layout
    CURRENT_LEN=$(max_length \
        "$REPORT_TITLE" \
        "$os_name" \
        "$os_kernel" \
        "$net_hostname" \
        "$net_machine_ip" \
        "$net_client_ip" \
        "$net_user" \
        "$cpu_model" \
        "$cpu_cores_per_socket vCPU(s) / $cpu_sockets Socket(s)" \
        "$cpu_hypervisor" \
        "$cpu_freq GHz" \
        "$cpu_1min_bar" \
        "$cpu_5min_bar" \
        "$cpu_15min_bar" \
        "$disk_used_gb/$disk_total_gb GiB [$disk_percent%]" \
        "$disk_bar" \
        "$disk_health" \
        "${mem_used_gb}/${mem_total_gb} GiB [${mem_percent}%]" \
        "${mem_bar}" \
        "$last_login_time" \
        "$last_login_ip" \
        "$sys_uptime" \
    )

    # Print header
    print_decorated_header
    print_centered_data "$REPORT_TITLE"
    print_centered_data "$APP_NAME"
    print_divider "top"

    # OS section
    print_data "OS" "$os_name"
    print_data "KERNEL" "$os_kernel"
    print_divider

    # Network section
    print_data "HOSTNAME" "$net_hostname"
    print_data "MACHINE IP" "$net_machine_ip"
    print_data "CLIENT  IP" "$net_client_ip"

    # DNS IPs
    dns_num=0
    for dns_ip in $net_dns_ip; do
        dns_num=$(( dns_num + 1 ))
        print_data "DNS  IP $dns_num" "$dns_ip"
    done

    print_data "USER" "$net_user"
    print_divider

    # CPU section
    print_data "PROCESSOR" "$cpu_model"
    print_data "CORES" "$cpu_cores_per_socket vCPU(s) / $cpu_sockets Socket(s)"
    print_data "HYPERVISOR" "$cpu_hypervisor"
    print_data "CPU FREQ" "$cpu_freq GHz"
    print_bar "LOAD  1m" "$cpu_1min_bar"
    print_bar "LOAD  5m" "$cpu_5min_bar"
    print_bar "LOAD 15m" "$cpu_15min_bar"

    # Disk section
    print_divider
    print_data "VOLUME" "$disk_used_gb/$disk_total_gb GiB [$disk_percent%]"
    print_bar "DISK USAGE" "$disk_bar"
    if [ "$zfs_present" -eq 1 ]; then
        print_data "ZFS HEALTH" "$disk_health"
    fi

    # Memory section
    print_divider
    print_data "MEMORY" "${mem_used_gb}/${mem_total_gb} GiB [${mem_percent}%]"
    print_bar "USAGE" "${mem_bar}"

    # Login/uptime section
    print_divider
    print_data "LAST LOGIN" "$last_login_time"
    if [ "$last_login_ip_present" -eq 1 ]; then
        print_data "" "$last_login_ip"
    fi
    print_data "UPTIME" "$sys_uptime"
    print_divider "end"
}

# Layout helper functions
print_decorated_header() {
    length=$((CURRENT_LEN+MAX_NAME_LEN+BORDERS_AND_PADDING))
    top="┌"
    bottom="├"
    i=0
    while [ $i -lt $(( length -2 )) ]; do
        top="${top}┬"
        bottom="${bottom}┴"
        i=$(( i + 1 ))
    done
    top="${top}┐"
    bottom="${bottom}┤"
    printf '%s\n' "$top"
    printf '%s\n' "$bottom"
}

print_centered_data() {
    max_len=$((CURRENT_LEN+MAX_NAME_LEN-BORDERS_AND_PADDING))
    text="$1"
    total_width=$((max_len + 12))
    text_len=${#text}
    padding_left=$(( (total_width - text_len) / 2 ))
    padding_right=$(( total_width - text_len - padding_left ))
    printf "│%${padding_left}s%s%${padding_right}s│\n" "" "$text" ""
}

print_divider() {
    side="$1"
    case "$side" in
        "top")
            left_symbol="├"
            middle_symbol="┬"
            right_symbol="┤"
            ;;
        "bottom")
            left_symbol="├"
            middle_symbol="┴"
            right_symbol="┤"
            ;;
        "end")
            left_symbol="└"
            middle_symbol="┴"
            right_symbol="┘"
            ;;
        *)
            left_symbol="├"
            middle_symbol="┼"
            right_symbol="┤"
            ;;
    esac
    length=$((CURRENT_LEN+MAX_NAME_LEN+BORDERS_AND_PADDING))
    divider="$left_symbol"
    i=0
    while [ $i -lt $(( length - 3 )) ]; do
        divider="${divider}─"
        if [ "$i" -eq $(( MAX_NAME_LEN + 1 )) ]; then
            divider="${divider}$middle_symbol"
        fi
        i=$(( i + 1 ))
    done
    divider="${divider}$right_symbol"
    printf '%s\n' "$divider"
}

print_data() {
    name="$1"
    data="$2"
    max_data_len=$CURRENT_LEN
    name_len=${#name}
    if [ "$name_len" -lt "$MIN_NAME_LEN" ]; then
        name=$(printf "%-${MIN_NAME_LEN}s" "$name")
    elif [ "$name_len" -gt "$MAX_NAME_LEN" ]; then
        name=$(echo "$name" | cut -c 1-$((MAX_NAME_LEN-1)))…
    else
        name=$(printf "%-${MAX_NAME_LEN}s" "$name")
    fi
    data_len=${#data}
    if [ "$data_len" -gt "$MAX_DATA_LEN" ] || [ "$data_len" -eq $(( MAX_DATA_LEN - 1 )) ]; then
        data=$(echo "$data" | cut -c 1-$(( MAX_DATA_LEN - 1 )))…
    else
        data=$(printf "%-${max_data_len}s" "$data")
    fi
    printf "│ %-${MAX_NAME_LEN}s │ %s │\n" "$name" "$data"
}

print_bar() {
    name="$1"
    data="$2"
    name_len=${#name}
    if [ "$name_len" -lt "$MIN_NAME_LEN" ]; then
        name=$(printf "%-${MIN_NAME_LEN}s" "$name")
    elif [ "$name_len" -gt "$MAX_NAME_LEN" ]; then
        name=$(echo "$name" | cut -c 1-$((MAX_NAME_LEN-1)))…
    else
        name=$(printf "%-${MAX_NAME_LEN}s" "$name")
    fi
    printf "│ %-${MAX_NAME_LEN}s │ %s │\n" "$name" "$data"
}

# ============================================================================
# MAIN
# ============================================================================

main() {
    if [ "$CONTINUOUS_MODE" -eq 1 ]; then
        # Continuous mode - loop forever
        while true; do
            # Collect all data
            os_data=$(collect_os_info)
            net_data=$(collect_network_info)
            cpu_data=$(collect_cpu_info)
            mem_data=$(collect_memory_info)
            disk_data=$(collect_disk_info)
            login_data=$(collect_login_info)

            # Parse into individual variables
            os_name=$(echo "$os_data" | sed -n '1p')
            os_kernel=$(echo "$os_data" | sed -n '2p')

            net_hostname=$(echo "$net_data" | sed -n '1p')
            net_machine_ip=$(echo "$net_data" | sed -n '2p')
            net_client_ip=$(echo "$net_data" | sed -n '3p')
            net_dns_ip=$(echo "$net_data" | sed -n '4p')
            net_user=$(echo "$net_data" | sed -n '5p')

            cpu_model=$(echo "$cpu_data" | sed -n '1p')
            cpu_cores_per_socket=$(echo "$cpu_data" | sed -n '2p')
            cpu_sockets=$(echo "$cpu_data" | sed -n '3p')
            cpu_hypervisor=$(echo "$cpu_data" | sed -n '4p')
            cpu_freq=$(echo "$cpu_data" | sed -n '5p')
            load_avg_1min=$(echo "$cpu_data" | sed -n '6p')
            load_avg_5min=$(echo "$cpu_data" | sed -n '7p')
            load_avg_15min=$(echo "$cpu_data" | sed -n '8p')
            cpu_cores=$(echo "$cpu_data" | sed -n '9p')

            mem_total=$(echo "$mem_data" | sed -n '1p')
            mem_available=$(echo "$mem_data" | sed -n '2p')
            mem_used=$(echo "$mem_data" | sed -n '3p')
            mem_percent=$(echo "$mem_data" | sed -n '4p')
            mem_total_gb=$(echo "$mem_data" | sed -n '5p')
            mem_used_gb=$(echo "$mem_data" | sed -n '6p')

            zfs_present=$(echo "$disk_data" | sed -n '1p')
            disk_total_gb=$(echo "$disk_data" | sed -n '2p')
            disk_used_gb=$(echo "$disk_data" | sed -n '3p')
            disk_percent=$(echo "$disk_data" | sed -n '4p')
            disk_health=$(echo "$disk_data" | sed -n '5p')

            last_login_time=$(echo "$login_data" | sed -n '1p')
            last_login_ip_present=$(echo "$login_data" | sed -n '2p')
            last_login_ip=$(echo "$login_data" | sed -n '3p')
            sys_uptime=$(echo "$login_data" | sed -n '4p')

            # Clear screen and render
            clear
            render_ui "$os_name" "$os_kernel" "$net_hostname" "$net_machine_ip" "$net_client_ip" "$net_dns_ip" "$net_user" \
                     "$cpu_model" "$cpu_cores_per_socket" "$cpu_sockets" "$cpu_hypervisor" "$cpu_freq" \
                     "$load_avg_1min" "$load_avg_5min" "$load_avg_15min" "$cpu_cores" \
                     "$mem_total" "$mem_available" "$mem_used" "$mem_percent" "$mem_total_gb" "$mem_used_gb" \
                     "$zfs_present" "$disk_total_gb" "$disk_used_gb" "$disk_percent" "$disk_health" \
                     "$last_login_time" "$last_login_ip_present" "$last_login_ip" "$sys_uptime"

            sleep "$REFRESH_RATE"
        done
    else
        # Single run mode
        os_data=$(collect_os_info)
        net_data=$(collect_network_info)
        cpu_data=$(collect_cpu_info)
        mem_data=$(collect_memory_info)
        disk_data=$(collect_disk_info)
        login_data=$(collect_login_info)

        # Parse into individual variables
        os_name=$(echo "$os_data" | sed -n '1p')
        os_kernel=$(echo "$os_data" | sed -n '2p')

        net_hostname=$(echo "$net_data" | sed -n '1p')
        net_machine_ip=$(echo "$net_data" | sed -n '2p')
        net_client_ip=$(echo "$net_data" | sed -n '3p')
        net_dns_ip=$(echo "$net_data" | sed -n '4p')
        net_user=$(echo "$net_data" | sed -n '5p')

        cpu_model=$(echo "$cpu_data" | sed -n '1p')
        cpu_cores_per_socket=$(echo "$cpu_data" | sed -n '2p')
        cpu_sockets=$(echo "$cpu_data" | sed -n '3p')
        cpu_hypervisor=$(echo "$cpu_data" | sed -n '4p')
        cpu_freq=$(echo "$cpu_data" | sed -n '5p')
        load_avg_1min=$(echo "$cpu_data" | sed -n '6p')
        load_avg_5min=$(echo "$cpu_data" | sed -n '7p')
        load_avg_15min=$(echo "$cpu_data" | sed -n '8p')
        cpu_cores=$(echo "$cpu_data" | sed -n '9p')

        mem_total=$(echo "$mem_data" | sed -n '1p')
        mem_available=$(echo "$mem_data" | sed -n '2p')
        mem_used=$(echo "$mem_data" | sed -n '3p')
        mem_percent=$(echo "$mem_data" | sed -n '4p')
        mem_total_gb=$(echo "$mem_data" | sed -n '5p')
        mem_used_gb=$(echo "$mem_data" | sed -n '6p')

        zfs_present=$(echo "$disk_data" | sed -n '1p')
        disk_total_gb=$(echo "$disk_data" | sed -n '2p')
        disk_used_gb=$(echo "$disk_data" | sed -n '3p')
        disk_percent=$(echo "$disk_data" | sed -n '4p')
        disk_health=$(echo "$disk_data" | sed -n '5p')

        last_login_time=$(echo "$login_data" | sed -n '1p')
        last_login_ip_present=$(echo "$login_data" | sed -n '2p')
        last_login_ip=$(echo "$login_data" | sed -n '3p')
        sys_uptime=$(echo "$login_data" | sed -n '4p')

        render_ui "$os_name" "$os_kernel" "$net_hostname" "$net_machine_ip" "$net_client_ip" "$net_dns_ip" "$net_user" \
                 "$cpu_model" "$cpu_cores_per_socket" "$cpu_sockets" "$cpu_hypervisor" "$cpu_freq" \
                 "$load_avg_1min" "$load_avg_5min" "$load_avg_15min" "$cpu_cores" \
                 "$mem_total" "$mem_available" "$mem_used" "$mem_percent" "$mem_total_gb" "$mem_used_gb" \
                 "$zfs_present" "$disk_total_gb" "$disk_used_gb" "$disk_percent" "$disk_health" \
                 "$last_login_time" "$last_login_ip_present" "$last_login_ip" "$sys_uptime"
    fi
}

main
