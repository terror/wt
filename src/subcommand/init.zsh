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

      local dir

      dir=$(command wt "$@") || return $?

      if [ -n "$dir" ]; then
        builtin cd "$dir" || return $?
      fi

      ;;
    *)
      command wt "$@"
      ;;
  esac
}
