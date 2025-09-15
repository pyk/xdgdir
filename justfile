set dotenv-load

repomix subcommand:
    #!/usr/bin/env bash
    if [[ {{subcommand}} == "xdgdir" ]] then
        npx repomix@latest . --style xml \
            -o repomix-xdgdir-$(date +%Y%m%d-%H%M%S).xml
    fi
