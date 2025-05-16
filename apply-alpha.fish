#!/usr/bin/env fish

function apply-alpha
    if test (count $argv) -ne 1
        echo "Usage: apply-alpha.fish <input.png>"
        return 1
    end

    set input $argv[1]
    set alpha 0.02
    set output (string replace -r '.png' "\-$alpha.png" $input)

    magick $input -alpha set -channel A -evaluate multiply $alpha $output
end

apply-alpha $argv
