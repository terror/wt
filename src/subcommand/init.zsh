wt() {
  case "$1" in
    create|c|remove|r|switch|s)
      local arg

      for arg in "$@"; do
        case "$arg" in
          -h|--help)
            command wt "$@"
            return $?
            ;;
          --)
            break
            ;;
        esac
      done

      local dir exit_code=0

      dir=$(
        command wt "$@" || exit_code=$?
        printf .
        exit "$exit_code"
      ) || exit_code=$?

      dir=${dir%.}
      dir=${dir%$'\n'}

      if [ -n "$dir" ]; then
        builtin cd "$dir" || return $?
      fi

      return "$exit_code"

      ;;
    *)
      command wt "$@"
      ;;
  esac
}
