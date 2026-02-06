wt() {
  if [ "$1" = "create" ] || [ "$1" = "c" ] || [ "$1" = "remove" ] || [ "$1" = "r" ] || [ "$1" = "switch" ] || [ "$1" = "s" ]; then
    local dir
    dir=$(command wt "$@") || return $?
    if [ -n "$dir" ]; then
      builtin cd "$dir" || return $?
    fi
  else
    command wt "$@"
  fi
}
