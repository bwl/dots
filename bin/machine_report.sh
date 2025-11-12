#!/bin/sh
# SPDX-License-Identified: BSD-3-Clause
#
# TR-101 Machine Report
# Copyright © 2024, U.S. Graphics, LLC. BSD-3-Clause License.
# Copyright © 2025, Dmitry Achkasov <achkasov.dmitry@live.com>.

# Global variables
MIN_NAME_LEN=5
MAX_NAME_LEN=10
MIN_DATA_LEN=5
MAX_DATA_LEN=32
BORDERS_AND_PADDING=7

# Basic configuration, change as needed
report_title="RETRO-OS PHASE 0"
app_name="TR-101 MACHINE REPORT"
last_login_ip_present=0
zfs_present=0
# zfs_filesystem="zroot/ROOT/os"
if command -v zpool >/dev/null 2>&1; then
    zfs_filesystem=$( zpool list -H -o name | tail -n 1 )
fi

debug() {
    # Uncomment for debug logging
    # printf "DEBUG: $@\n"
    printf ""
}

# Utilities
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

# All data strings must go here
set_current_len() {
    CURRENT_LEN=$(max_length                                     \
        "$barcode_header"                                        \
        "$report_title"                                          \
        "$os_name"                                               \
        "$os_kernel"                                             \
        "$net_hostname"                                          \
        "$net_machine_ip"                                        \
        "$net_client_ip"                                         \
        "$net_current_user"                                      \
        "$cpu_model"                                             \
        "$cpu_cores_per_socket vCPU(s) / $cpu_sockets Socket(s)" \
        "$cpu_hypervisor"                                        \
        "$cpu_freq GHz"                                          \
        "$cpu_1min_bar_graph"                                    \
        "$cpu_5min_bar_graph"                                    \
        "$cpu_15min_bar_graph"                                   \
        "$zfs_used_gb/$zfs_available_gb GiB [$disk_percent%]"    \
        "$disk_bar_graph"                                        \
        "$zfs_health"                                            \
        "$root_used_gb/$root_total_gb GiB [$disk_percent%]"      \
        "${mem_used_gb}/${mem_total_gb} GiB [${mem_percent}%]"   \
        "${mem_bar_graph}"                                       \
        "$last_login_time"                                       \
        "$last_login_ip"                                         \
        "$sys_uptime"                                            \
    )
}

PRINT_DECORATED_HEADER() {
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

PRINT_HEADER() {
    length=$((CURRENT_LEN+MAX_NAME_LEN+BORDERS_AND_PADDING))
    top="┌"
    i=0
    while [ $i -lt $(( length -2 )) ]; do
        top="${top}─"
        i=$(( i + 1 ))
    done
    top="${top}┐"

    printf '%s\n' "$top"
}

PRINT_CENTERED_DATA() {
    max_len=$((CURRENT_LEN+MAX_NAME_LEN-BORDERS_AND_PADDING))
    text="$1"
    total_width=$((max_len + 12))

    text_len=${#text}
    padding_left=$(( (total_width - text_len) / 2 ))
    padding_right=$(( total_width - text_len - padding_left ))

    printf "│%${padding_left}s%s%${padding_right}s│\n" "" "$text" ""
}

PRINT_DIVIDER() {
    # either "top" or "bottom", no argument means middle divider
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

PRINT_DATA() {
    name="$1"
    data="$2"
    max_data_len=$CURRENT_LEN

    # Pad name
    name_len=${#name}
    if [ "$name_len" -lt "$MIN_NAME_LEN" ]; then
        name=$(printf "%-${MIN_NAME_LEN}s" "$name")
    elif [ "$name_len" -gt "$MAX_NAME_LEN" ]; then
        name=$(echo "$name" | cut -c 1-$((MAX_NAME_LEN-1)))…
    else
        name=$(printf "%-${MAX_NAME_LEN}s" "$name")
    fi

    # Truncate or pad data
    data_len=${#data}
    if [ "$data_len" -gt "$MAX_DATA_LEN" ] || [ "$data_len" -eq $(( MAX_DATA_LEN - 1 )) ]; then
        data=$(echo "$data" | cut -c 1-$(( MAX_DATA_LEN - 1 )))…
    else
    	# TODO: stupid Debian `dash` cannot into UTF-8 and might trim strings earlier
        data=$(printf "%-${max_data_len}s" "$data")
    fi

    printf "│ %-${MAX_NAME_LEN}s │ %s │\n" "$name" "$data"
}


PRINT_BAR() {
    name="$1"
    data="$2"
    max_data_len=$CURRENT_LEN

    # Pad name
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


PRINT_FOOTER() {
    length=$((CURRENT_LEN+MAX_NAME_LEN+BORDERS_AND_PADDING))
    bottom="└"
    i=0
    while [ $i -lt $(( length -2 )) ]; do
        bottom="${bottom}─"
        i=$(( i + 1 ))
    done
    bottom="${bottom}┘"
    printf '%s\n' "$bottom"
}

bar_graph() {
    percent=0
    num_blocks=0
    width=$CURRENT_LEN
    graph=""
    used=$1
    total=$2

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

#    i=0
    while [ "$i" -lt "$width" ]; do
         graph="${graph}░"
        i=$(( i + 1 ))
    done

    printf "%s" "${graph}"
}

get_ip_addr() {
    # Initialize variables
    ipv4_address=""
    ipv6_address=""

    # Check if ifconfig command exists
    if command -v ifconfig >/dev/null 2>&1; then
        # Try to get IPv4 address using ifconfig
        ipv4_address=$(ifconfig | awk '
            /^[a-z]/ {iface=$1}
            iface != "lo:" && iface !="lo0:" && iface !~ /^docker/ && /inet / && !found_ipv4 {found_ipv4=1; print $2}')

        # If IPv4 address not available, try IPv6 using ifconfig
        if [ -z "$ipv4_address" ]; then
            ipv6_address=$(ifconfig | awk '
                /^[a-z]/ {iface=$1}
                iface != "lo:" && iface != "lo0:" && iface !~ /^docker/ && /inet6 / && !found_ipv6 {found_ipv6=1; print $2}')
        fi
    elif command -v ip >/dev/null 2>&1; then
        # Try to get IPv4 address using ip addr
        ipv4_address=$(ip -o -4 addr show | awk '
            $2 != "lo" && $2 !~ /^docker/ {split($4, a, "/"); if (!found_ipv4++) print a[1]}')

        # If IPv4 address not available, try IPv6 using ip addr
        if [ -z "$ipv4_address" ]; then
            ipv6_address=$(ip -o -6 addr show | awk '
                $2 != "lo" && $2 !~ /^docker/ {split($4, a, "/"); if (!found_ipv6++) print a[1]}')
        fi
    fi

    # If neither IPv4 nor IPv6 address is available, assign "No IP found"
    if [ -z "$ipv4_address" ] && [ -z "$ipv6_address" ]; then
        ip_address="No IP found"
    else
        # Prioritize IPv4 if available, otherwise use IPv6
        ip_address="${ipv4_address:-$ipv6_address}"
    fi

    printf '%s' "$ip_address"
}

debug "COLLECTING OS INFO"
# Operating System Information
os_name="???"
if [ -f /etc/os-release ]; then
#    . /etc/os-release
    NAME="$( grep "^NAME=" /etc/os-release | cut -d'=' -f2 | tr -d '"')"
    debug "OS: $NAME"
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

debug "COLLECTING NET INFO"
# Network Information
net_current_user=$(whoami)
if ! [ "$(command -v hostname)" ]; then
    net_hostname=$(grep -w "$(uname -n)" /etc/hosts | awk '{print $2}' | head -n 1)
else
    net_hostname=$(hostname)
fi

if [ -z "$net_hostname" ]; then net_hostname="Not Defined"; fi

net_machine_ip=$(get_ip_addr)
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


debug "COLLECTING CPU INFO"
# CPU Information

case "$(uname)" in
    SunOS)
        cpu_model="$(kstat -C -m cpu_info -i 0 -s brand | cut -f 5 -d ':')"
        cpu_cores_per_socket="$(psrinfo -tc)"
        cpu_sockets="$(psrinfo -p)"
        if [ "$(smbios -t SMB_TYPE_SYSTEM | grep "Product" | tr -d ' ' |  cut -d ':' -f2)" = "VirtualMachine" ]; then
            cpu_hypervisor=$(smbios -t SMB_TYPE_BIOS | grep "Version String" | cut -d ':' -f2 | sed 's/^[[:space:]]* //')
        fi
        cpu_cores="$(nproc --all)"
        ;;
    Darwin)
        cpu_model="$(sysctl -n machdep.cpu.brand_string)"
        cpu_cores_per_socket="$(sysctl -n machdep.cpu.core_count)"
        cpu_sockets="$(sysctl -n hw.physicalcpu)"
        cpu_cores="$(sysctl -n hw.ncpu)"
        ;;
    *)
    	if ! command -v lscpu >/dev/null 2>&1; then
		printf "ERROR: \`lscpu\` utility is not found"
		exit 1
	fi
    	if ! command -v nproc >/dev/null 2>&1; then
		printf "ERROR: \`nproc\` utility is not found"
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



case "$(uname)" in
  Linux)
    cpu_freq="$(grep 'cpu MHz' /proc/cpuinfo | cut -f 2 -d ':' | awk 'NR==1 { printf "%.2f", $1 / 1000 }')" # Convert from M to G units
    ;;
  Darwin)
     # Intel Macs expose hw.cpufrequency, Apple Silicon does not
     cpu_freq="$(sysctl -n hw.cpufrequency 2>/dev/null | awk 'NR==1 { printf "%.2f", $1 / 1000000000 }')"
     if [ -z "$cpu_freq" ]; then
         # Try hw.cpufrequency_max for some Intel systems
         cpu_freq="$(sysctl -n hw.cpufrequency_max 2>/dev/null | awk 'NR==1 { printf "%.2f", $1 / 1000000000 }')"
     fi
     # Apple Silicon (M1/M2/M3) doesn't expose frequency via sysctl - show "N/A"
     if [ -z "$cpu_freq" ] || [ "$cpu_freq" = "0.00" ]; then
         cpu_freq="N/A"
     fi
     ;;
  FreeBSD)
    cpu_freq="$(sysctl -n dev.cpu.0.freq | awk 'NR==1 { printf "%.2f", $1 / 1000 }')" # Convert from M to G units
      ;;
  SunOS)
     cpu_freq="$(kstat -C -m cpu_info -i 0 -s clock_MHz | cut -f 5 -d ':' | awk 'NR==1 { printf "%.2f", $1 / 1000 }')" # Convert from M to G units
    ;;
  *)
      cpu_freq="???"
      ;;
esac

# Linux, Solaris, illumos use "load average:"
# MacOS, FreeBSD use "load averages:"
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

debug "COLLECTING MEMORY INFO"
# Memory Information
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
mem_percent=$(printf "%.2f" "$mem_percent")
mem_total_gb=$(echo "$mem_total" | awk '{ printf "%.2f", $1 / (1024 * 1024) }') # (From Ki to Gi units)
# mem_available_gb=$(echo "$mem_available" | awk '{ printf "%.2f", $1 / (1024 * 1024) }') # (From Ki to Gi units) Not used currently
mem_used_gb=$(echo "$mem_used" | awk '{ printf "%.2f", $1 / (1024 * 1024) }')


debug "COLLECTING DISK INFO"
# Disk Information
if command -v zfs >/dev/null 2>&1 && [ "$(( $(zpool list -H | wc -l) ))" -gt 0 ]; then
    zfs_present=1
    zfs_health=$(zpool status -x "$zfs_filesystem" | grep -q "is healthy" && echo  "HEALTH O.K.")
    zfs_available=$(zfs get -o value -Hp available "$zfs_filesystem")
    zfs_used=$(zfs get -o value -Hp used "$zfs_filesystem")
    zfs_available_gb=$(echo "$zfs_available" | awk '{ printf "%.2f", $1 / (1024 * 1024 * 1024) }') # (To G units)
    zfs_used_gb=$(echo "$zfs_used" | awk '{ printf "%.2f", $1 / (1024 * 1024 * 1024) }') # (To G units)
    disk_percent=$(awk -v used="$zfs_used" -v available="$zfs_available" 'BEGIN { printf "%.2f", (used / available) * 100 }')
else
    # Thanks https://github.com/AnarchistHoneybun
    root_partition="/"
    root_used=$(df -m "$root_partition" | awk 'NR==2 {print $3}')
    root_total=$(df -m "$root_partition" | awk 'NR==2 {print $2}')
    root_total_gb=$(awk -v total="$root_total" 'BEGIN { printf "%.2f", total / 1024 }')
    root_used_gb=$(awk -v used="$root_used" 'BEGIN { printf "%.2f", used / 1024 }')
    disk_percent=$(awk -v used="$root_used" -v total="$root_total" 'BEGIN { printf "%.2f", (used / total) * 100 }')
fi

debug "COLLECTING LOGIN INFO"
# Last login and Uptime
last_login_time="Never logged in"
if command -v lastlog >/dev/null 2>&1; then
    last_login=$(lastlog -u "$USER")
    last_login_ip=$(echo "$last_login" | awk 'NR==2 {print $3}')
    last_login_ip=$( echo "$last_login_ip" | sed -n '/^[0-9]\{1,3\}\(\.[0-9]\{1,3\}\)\{3\}$/p' )

    if [ -z "$last_login_ip" ]; then
        last_login_ip_present=0
        debug "COLLECTING LOGIN IP: NOT PRESENT: $last_login_ip_present"
    else
        if echo "$last_login_ip" | awk -F. 'NF==4 && $1<=255 && $2<=255 && $3<=255 && $4<=255' >/dev/null 2>&1; then
            last_login_ip_present=1
            last_login_time=$(echo "$last_login" | awk 'NR==2 {print $6, $7, $10, $8}')
            debug "COLLECTING LOGIN IP: IDENTIFIED: $last_login_ip_present, $last_login_time"
        else
            last_login_time=$(echo "$last_login" | awk 'NR==2 {print $4, $5, $8, $6}')
                debug "COLLECTING LOGIN IP: IDENTIFIED: $last_login_ip_present, $last_login_time"
        fi
    fi
else
    debug "COLLECTING LOGIN INFO - NO LASTLOG"
    case $(uname) in
        FreeBSD)
            last_login_ip=$(lastlogin --libxo json,pretty "$USER" | awk -F'"' '/"from"/ {print $4}')
            if [ -n "$last_login_ip" ]; then
                last_login_ip_present=1
            fi
            last_login_time=$(lastlogin --libxo json,pretty "$USER" | awk -F '"' '/"login-time"/ {print $4}')
            debug "COLLECTING LOGIN INFO - LASTLOGIN: $last_login_time"
            ;;

        Darwin)
            # macOS 'last' format: username tty [day] month date time [- time] [(duration)]
            last_login=$(last -1 "$USER" | head -n 1)
            # Check if line contains actual login info (not 'reboot' or 'shutdown')
            if echo "$last_login" | grep -q "^$USER"; then
                # Field 3 could be IP/hostname or day-of-week, check if it's an IP
                last_login_ip=$(echo "$last_login" | awk '{print $3}')
                if echo "$last_login_ip" | awk -F. 'NF==4 && $1<=255 && $2<=255 && $3<=255 && $4<=255' >/dev/null 2>&1; then
                    last_login_ip_present=1
                    # With IP: username tty IP day month date time
                    last_login_time=$(echo "$last_login" | awk '{print $4, $5, $6, $7}')
                else
                    # No IP: username tty day month date time (no IP field)
                    # Fields: $3=day $4=month $5=date $6=time
                    last_login_time=$(echo "$last_login" | awk '{print $4, $5, $6}')
                fi
            else
                last_login_time="Never logged in"
            fi
            debug "COLLECTING LOGIN INFO - DARWIN LAST: $last_login_time"
            ;;
        *)
            last_login=$(last "$USER" | head -n 1)
            last_login_ip=$(echo "$last_login" | awk 'NR==1 {print $3}')
            debug "COLLECTING LOGIN INFO - LAST: $last_login, $last_login_ip"
            if echo "$last_login_ip" | awk -F. 'NF==4 && $1<=255 && $2<=255 && $3<=255 && $4<=255' >/dev/null 2>&1; then
                last_login_ip_present=1
                last_login_time=$(echo "$last_login" | awk 'NR==1 {print $4, $5, $6, $7}')
            debug "COLLECTING LOGIN IP: IDENTIFIED: $last_login_ip_present, $last_login_time"
            else
                last_login_time=$(echo "$last_login" | awk 'NR==1 {print $3, $4, $5, $6}')
                debug "COLLECTING LOGIN IP: IDENTIFIED: $last_login_ip_present, $last_login_time"
            fi

            ;;
    esac
fi


# if [[ "$last_login_ip" =~ ^[0-9]+\.[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
#     last_login_ip_present=1
#     last_login_time=$(echo "$last_login" | awk 'NR==2 {print $6, $7, $10, $8}')
# else
#     last_login_time=$(echo "$last_login" | awk 'NR==2 {print $4, $5, $8, $6}')
#     # Check for **Never logged in** edge case
#     if [ "$last_login_time" = "in**" ]; then
#         last_login_time="Never logged in"
#     fi
# fi

# sys_uptime=$(uptime | cut -d',' -f1 | sed 's/^[^ ]* //; s/up\s*//; s/\s*day\(s*\)/d/; s/\s*hour\(s*\)/h/; s/\s*minute\(s*\)/m/')
sys_uptime=$(uptime | cut -d',' -f1 \
                    | sed 's/^[^ ]* //' \
                    | sed 's/^[^ ]* //' \
                    | sed 's/^[ ]* //' \
                    | sed "s/up[ ][[:space:]]*//" \
                    | sed 's/[[:space:]]*day\(s*\)/d/' \
                    | sed 's/[[:space:]]*hour\(s*\)/h/' \
                    | sed 's/[[:space:]]*minute\(s*\)/m/')


debug "PREPARING GRAPHS"
# Set current length before graphs get calculated

barcode_header="█▐ ▌ ▐▌█▌ ▌█ ▐▐█▐ █▐█ ▌▐█ █▐ █▌█ ▌▐█▐▌▐█▐ █ █ ▐█▐▐▌"

set_current_len


debug "PREPARING GRAPHS - CPU"
# Create graphs
debug "PREPARING GRAPHS - CPU 1 MIN LOAD: $load_avg_1min, $cpu_cores"
cpu_1min_bar_graph=$(bar_graph "$load_avg_1min" "$cpu_cores")
debug "PREPARING GRAPHS - CPU 5 MIN LOAD"
cpu_5min_bar_graph=$(bar_graph "$load_avg_5min" "$cpu_cores")
debug "PREPARING GRAPHS - CPU 15 MIN LOAD"
cpu_15min_bar_graph=$(bar_graph "$load_avg_15min" "$cpu_cores")

debug "PREPARING GRAPHS - MEMORY"
mem_bar_graph=$(bar_graph "$mem_used" "$mem_total")

debug "PREPARING GRAPHS - DISK"
if [ $zfs_present -eq 1 ]; then
    debug "PREPARING GRAPHS - DISK ZFS: $zfs_used"
    disk_bar_graph=$(bar_graph "$zfs_used" "$zfs_available")
else
    debug "PREPARING GRAPHS - DISK REGULAR"
    disk_bar_graph=$(bar_graph "$root_used" "$root_total")
fi


# Machine Report
# PRINT_HEADER
PRINT_DECORATED_HEADER
# PRINT_HEADER
# PRINT_CENTERED_DATA "$barcode_header"
PRINT_CENTERED_DATA "$report_title"
PRINT_CENTERED_DATA "$app_name"
PRINT_DIVIDER "top"
PRINT_DATA "OS" "$os_name"
PRINT_DATA "KERNEL" "$os_kernel"
PRINT_DIVIDER
PRINT_DATA "HOSTNAME" "$net_hostname"
PRINT_DATA "MACHINE IP" "$net_machine_ip"
PRINT_DATA "CLIENT  IP" "$net_client_ip"

# TODO: verify
dns_num=0
for dns_ip in $net_dns_ip; do
    dns_num=$(( dns_num + 1 ))
    PRINT_DATA "DNS  IP $dns_num" "$dns_ip"
done

PRINT_DATA "USER" "$net_current_user"
PRINT_DIVIDER
PRINT_DATA "PROCESSOR" "$cpu_model"
PRINT_DATA "CORES" "$cpu_cores_per_socket vCPU(s) / $cpu_sockets Socket(s)"
PRINT_DATA "HYPERVISOR" "$cpu_hypervisor"
PRINT_DATA "CPU FREQ" "$cpu_freq GHz"
PRINT_BAR "LOAD  1m" "$cpu_1min_bar_graph"
PRINT_BAR "LOAD  5m" "$cpu_5min_bar_graph"
PRINT_BAR "LOAD 15m" "$cpu_15min_bar_graph"

if [ $zfs_present -eq 1 ]; then
    PRINT_DIVIDER
    PRINT_DATA "VOLUME" "$zfs_used_gb/$zfs_available_gb GiB [$disk_percent%]"
    PRINT_BAR "DISK USAGE" "$disk_bar_graph"
    PRINT_DATA "ZFS HEALTH" "$zfs_health"
else
    PRINT_DIVIDER
    PRINT_DATA "VOLUME" "$root_used_gb/$root_total_gb GiB [$disk_percent%]"
    PRINT_BAR "DISK USAGE" "$disk_bar_graph"
fi

PRINT_DIVIDER
PRINT_DATA "MEMORY" "${mem_used_gb}/${mem_total_gb} GiB [${mem_percent}%]"
PRINT_BAR "USAGE" "${mem_bar_graph}"
PRINT_DIVIDER
PRINT_DATA "LAST LOGIN" "$last_login_time"

if [ $last_login_ip_present -eq 1 ]; then
    PRINT_DATA "" "$last_login_ip"
fi

PRINT_DATA "UPTIME" "$sys_uptime"
PRINT_DIVIDER "end"
