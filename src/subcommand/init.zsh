wt() {
  if [ "$1" = "switch" ]; then
    local dir
    dir=$(command wt switch "${@:2}") || return $?
    if [ -n "$dir" ]; then
      builtin cd "$dir" || return $?
    fi
  else
    command wt "$@"
  fi
}
