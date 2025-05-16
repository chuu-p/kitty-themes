#!/usr/bin/env fish

function format-conf
    if test -z "$argv[1]"
        echo "Usage: format_conf.fish <config_file>"
        return 1
    end

    set config_file "$argv[1]"
    set temp_file (mktemp)

    awk '!/^#/ { key=$1; value=$2; for (i=3; i<=NF; i++) value = value " " $i; printf "%-30s %s\n", key, value; next } { print }' "$config_file" >"$temp_file"

    if test $status -eq 0
        mv "$temp_file" "$config_file"
        echo "Formatted: $config_file"
    else
        echo "Error during formatting."
        rm -f "$temp_file"
        return 1
    end
end

format-conf $argv
