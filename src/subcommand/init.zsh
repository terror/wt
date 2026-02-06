wt() {
  if [ "$1" = "create" ] || [ "$1" = "remove" ] || [ "$1" = "switch" ]; then
    local dir
    dir=$(command wt "$@") || return $?
    if [ -n "$dir" ]; then
      builtin cd "$dir" || return $?
    fi
  else
    command wt "$@"
  fi
}
