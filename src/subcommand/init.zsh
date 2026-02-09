wt() {
  case "$1" in
    convert|cv|create|c|remove|r|switch|s)
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
